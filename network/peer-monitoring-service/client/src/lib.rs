// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    logging::{LogEntry, LogEvent, LogSchema},
    network::NetworkInfoState,
};
use aptos_config::{config::PeerMonitoringServiceConfig, network_id::PeerNetworkId};
use aptos_id_generator::U64IdGenerator;
use aptos_infallible::RwLock;
use aptos_logger::{info, warn};
use aptos_network::application::{
    interface::NetworkClient, metadata::PeerMonitoringMetadata, storage::PeersAndMetadata,
};
use aptos_peer_monitoring_service_types::PeerMonitoringServiceMessage;
use aptos_time_service::{TimeService, TimeServiceTrait};
use client::PeerMonitoringServiceClient;
use error::Error;
use futures::StreamExt;
use latency::LatencyPingState;
use std::{collections::HashMap, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{runtime::Handle, task::JoinHandle};

mod client;
mod error;
mod latency;
mod logging;
mod metrics;
mod network;

/// A simple container that holds the state of the peer monitor
#[derive(Clone, Debug)]
pub struct PeerMonitorState {
    latency_ping_states: Arc<RwLock<HashMap<PeerNetworkId, LatencyPingState>>>, // Map of peers to latency ping states
    network_info_states: Arc<RwLock<HashMap<PeerNetworkId, NetworkInfoState>>>, // Map of peers to network info states
    request_id_generator: Arc<U64IdGenerator>, // Used for generating the next request/response id.
}

impl PeerMonitorState {
    pub fn new() -> Self {
        Self {
            latency_ping_states: Arc::new(RwLock::new(HashMap::new())),
            network_info_states: Arc::new(RwLock::new(HashMap::new())),
            request_id_generator: Arc::new(U64IdGenerator::new()),
        }
    }
}

impl Default for PeerMonitorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs the peer monitor that continuously monitors
/// the state of the peers.
pub async fn start_peer_monitor(
    peer_monitoring_config: PeerMonitoringServiceConfig,
    network_client: NetworkClient<PeerMonitoringServiceMessage>,
    time_service: TimeService,
    runtime: Option<Handle>,
) {
    // Create a new client and peer monitor state
    let peer_monitoring_client = PeerMonitoringServiceClient::new(network_client);
    let peer_monitor_state = PeerMonitorState::new();

    // Get the peers and metadata struct
    let peers_and_metadata = peer_monitoring_client.get_peers_and_metadata();

    // Spawns the updater for the peers and metadata
    spawn_peers_and_metadata_updater(
        peer_monitoring_config.clone(),
        peer_monitor_state.clone(),
        peers_and_metadata.clone(),
        time_service.clone(),
        runtime.clone(),
    );

    // Create an interval ticker for the monitor loop
    let peer_monitor_loop_duration =
        Duration::from_millis(peer_monitoring_config.peer_monitor_loop_interval_ms);
    let peer_monitor_loop_ticker = time_service.interval(peer_monitor_loop_duration);
    futures::pin_mut!(peer_monitor_loop_ticker);

    // Start the peer monitoring loop
    info!(LogSchema::new(LogEntry::PeerMonitorLoop)
        .event(LogEvent::StartedPeerMonitorLoop)
        .message("Starting the peer monitor!"));
    loop {
        // Wait for the next round before pinging peers
        peer_monitor_loop_ticker.next().await;

        // Get all connected peers
        let connected_peers_and_metadata =
            match peers_and_metadata.get_connected_peers_and_metadata() {
                Ok(connected_peers_and_metadata) => connected_peers_and_metadata,
                Err(error) => {
                    warn!(
                        (LogSchema::new(LogEntry::PeerMonitorLoop)
                            .event(LogEvent::UnexpectedErrorEncountered)
                            .error(&error.into())
                            .message("Failed to get connected peers and metadata!"))
                    );
                    continue; // Move to the new loop iteration
                },
            };

        // Send latency pings to the peers that need pinging
        latency::send_peer_latency_pings(
            peer_monitoring_config.clone(),
            time_service.clone(),
            runtime.clone(),
            peer_monitoring_client.clone(),
            &peer_monitor_state,
            connected_peers_and_metadata.keys().cloned().collect(),
        );

        // Send network info requests to the peers that need pinging
        network::send_network_info_requests(
            peer_monitoring_config.clone(),
            time_service.clone(),
            runtime.clone(),
            peer_monitoring_client.clone(),
            &peer_monitor_state,
            connected_peers_and_metadata.keys().cloned().collect(),
        );
    }
}

/// Spawns a task that continuously updates the peers and metadata
/// struct with the latest information stored for each peer.
fn spawn_peers_and_metadata_updater(
    peer_monitoring_config: PeerMonitoringServiceConfig,
    peer_monitor_state: PeerMonitorState,
    peers_and_metadata: Arc<PeersAndMetadata>,
    time_service: TimeService,
    runtime: Option<Handle>,
) -> JoinHandle<()> {
    // Create the updater task for the peers and metadata struct
    let metadata_updater = async move {
        // Create an interval ticker for the updater loop
        let metadata_updater_loop_duration =
            Duration::from_millis(peer_monitoring_config.metadata_updater_interval_ms);
        let metadata_updater_loop_ticker = time_service.interval(metadata_updater_loop_duration);
        futures::pin_mut!(metadata_updater_loop_ticker);

        // Start the updater loop
        info!(LogSchema::new(LogEntry::MetadataUpdaterLoop)
            .event(LogEvent::StartedMetadataUpdaterLoop)
            .message("Starting the peers and metadata updater!"));
        loop {
            // Wait for the next round before updating peers and metadata
            metadata_updater_loop_ticker.next().await;

            // Get all peers
            let all_peers = match peers_and_metadata.get_all_peers() {
                Ok(all_peers) => all_peers,
                Err(error) => {
                    warn!(LogSchema::new(LogEntry::UpdatePeerMonitoringMetadata)
                        .event(LogEvent::UnexpectedErrorEncountered)
                        .error(&error.into())
                        .message("Failed to get all peers!"));
                    continue; // Move to the new loop iteration
                },
            };

            // Update the latest peer monitoring metadata states
            for peer_network_id in all_peers {
                let peer_monitoring_metadata = get_latest_peer_monitoring_metadata(
                    peer_monitor_state.clone(),
                    &peer_network_id,
                );
                if let Err(error) = peers_and_metadata
                    .update_peer_monitoring_metadata(peer_network_id, peer_monitoring_metadata)
                {
                    warn!(LogSchema::new(LogEntry::UpdatePeerMonitoringMetadata)
                        .event(LogEvent::UnexpectedErrorEncountered)
                        .peer(&peer_network_id)
                        .error(&error.into()));
                }
            }
        }
    };

    // Spawn the peers and metadata updater task
    if let Some(runtime) = runtime {
        runtime.spawn(metadata_updater)
    } else {
        tokio::spawn(metadata_updater)
    }
}

/// Gets the latest peer monitoring metadata using the peer monitor state
fn get_latest_peer_monitoring_metadata(
    peer_monitor_state: PeerMonitorState,
    peer_network_id: &PeerNetworkId,
) -> PeerMonitoringMetadata {
    // Create an empty monitoring metadata entry for the peer
    let mut peer_monitoring_metadata = PeerMonitoringMetadata::default();

    // Get and store the average latency ping
    let average_latency_ping_secs = peer_monitor_state
        .latency_ping_states
        .read()
        .get(peer_network_id)
        .and_then(|latency_ping_state| latency_ping_state.get_average_latency_ping_secs());
    peer_monitoring_metadata.average_ping_latency_secs = average_latency_ping_secs;

    // Get and store the connected peers and metadata
    let network_info_response = peer_monitor_state
        .network_info_states
        .read()
        .get(peer_network_id)
        .and_then(|network_info_state| network_info_state.get_latest_network_info_response());
    let connected_peers_and_metadata = network_info_response
        .clone()
        .map(|network_info_response| network_info_response.connected_peers_and_metadata);
    peer_monitoring_metadata.connected_peers_and_metadata = connected_peers_and_metadata;

    // TODO: don't blindly trust the depth response!

    // Get and store the depth from the validators
    let depth_from_validators = network_info_response
        .map(|network_info_response| network_info_response.depth_from_validators);
    peer_monitoring_metadata.depth_from_validators = depth_from_validators;

    peer_monitoring_metadata
}
