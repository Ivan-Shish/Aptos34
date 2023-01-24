// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::logging::{LogEntry, LogEvent, LogSchema};
use aptos_config::{config::PeerMonitoringServiceConfig, network_id::PeerNetworkId};
use aptos_id_generator::U64IdGenerator;
use aptos_infallible::RwLock;
use aptos_logger::{info, warn};
use aptos_network::application::interface::NetworkClient;
use aptos_peer_monitoring_service_types::{
    LatencyPingRequest, PeerMonitoringServiceMessage, PeerMonitoringServiceRequest,
};
use aptos_time_service::{TimeService, TimeServiceTrait};
use error::Error;
use futures::StreamExt;
use latency::LatencyPingState;
use network::PeerMonitoringServiceClient;
use std::{collections::HashMap, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::runtime::Handle;

mod error;
mod latency;
mod logging;
mod metrics;
mod network;

/// A simple container that holds the state of the peer monitor
#[derive(Clone, Debug)]
struct PeerMonitorState {
    latency_ping_states: Arc<RwLock<HashMap<PeerNetworkId, LatencyPingState>>>, // Map of peers to latency ping states
    request_id_generator: Arc<U64IdGenerator>, // Used for generating the next request/response id.
}

impl PeerMonitorState {
    pub fn new() -> Self {
        Self {
            latency_ping_states: Arc::new(RwLock::new(HashMap::new())),
            request_id_generator: Arc::new(U64IdGenerator::new()),
        }
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
    // Create an interval ticker for the monitor loop
    let interval_duration =
        Duration::from_millis(peer_monitoring_config.peer_monitor_loop_interval_ms);
    let ticker = time_service.interval(interval_duration);
    futures::pin_mut!(ticker);

    // Create a new client and peer monitor state
    let peer_monitoring_client = PeerMonitoringServiceClient::new(network_client);
    let peer_monitor_state = PeerMonitorState::new();

    // Get the peers and metadata struct
    let peers_and_metadata = peer_monitoring_client.get_peers_and_metadata();

    // Start the monitor loop
    info!(
        (LogSchema::new(LogEntry::PeerMonitorLoop)
            .event(LogEvent::StartedPeerMonitorLoop)
            .message("Starting the peer monitor!"))
    );
    loop {
        // Wait for the next round before pinging peers
        ticker.next().await;

        // Get all connected peers
        let connected_peers_and_metadata =
            match peers_and_metadata.get_connected_peers_and_metadata() {
                Ok(connected_peers_and_metadata) => connected_peers_and_metadata,
                Err(error) => {
                    warn!(
                        (LogSchema::new(LogEntry::PeerMonitorLoop)
                            .event(LogEvent::UnexpectedErrorEncountered)
                            .error(&error.into())
                            .message("Failed to run the peer monitor loop!"))
                    );
                    continue; // Move to the new loop iteration
                },
            };

        // Ping the peers that need to be refreshed
        let mut num_in_flight_pings = 0;
        for peer_network_id in connected_peers_and_metadata.keys() {
            let should_ping_peer = match peer_monitor_state
                .latency_ping_states
                .read()
                .get(peer_network_id)
            {
                Some(latency_ping_state) => {
                    let latency_ping_interval_ms = peer_monitoring_config
                        .latency_monitoring
                        .latency_ping_interval_ms;

                    // If there's an in-flight ping, update the counter
                    if latency_ping_state.in_flight_latency_ping() {
                        num_in_flight_pings += 1;
                    }

                    // Check if the peer needs a new ping and there isn't currently one in-flight
                    latency_ping_state.needs_new_latency_ping(latency_ping_interval_ms)
                        && !latency_ping_state.in_flight_latency_ping()
                },
                None => true, // We should immediately ping the peer
            };

            // Only ping the peer if it needs to be pinged
            if should_ping_peer {
                if let Err(error) = latency::ping_peer(
                    peer_monitoring_config.latency_monitoring.clone(),
                    peer_monitoring_client.clone(),
                    *peer_network_id,
                    peer_monitor_state.request_id_generator.clone(),
                    peer_monitor_state.latency_ping_states.clone(),
                    time_service.clone(),
                    runtime.clone(),
                ) {
                    warn!(
                        (LogSchema::new(LogEntry::LatencyPing)
                            .event(LogEvent::UnexpectedErrorEncountered)
                            .peer(peer_network_id)
                            .error(&error))
                    );
                }
            }
        }

        // Update the in-flight metrics
        update_in_flight_metrics(num_in_flight_pings);
    }
}

/// Updates metrics for the number of in-flight pings
fn update_in_flight_metrics(num_in_flight_pings: u64) {
    // TODO: Avoid having to use a dummy request to get the label
    let latency_ping_label =
        PeerMonitoringServiceRequest::LatencyPing(LatencyPingRequest { ping_counter: 0 })
            .get_label();
    metrics::update_in_flight_pings(latency_ping_label, num_in_flight_pings);
}
