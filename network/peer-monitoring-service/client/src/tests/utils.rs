// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer_states::key_value::{PeerStateKey, StateValueInterface},
    start_peer_monitor_with_state,
    tests::mock::MockMonitoringServer,
    PeerMonitorState, PeerMonitoringServiceClient,
};
use aptos_config::{
    config::{
        LatencyMonitoringConfig, NetworkMonitoringConfig, NodeConfig, PeerMonitoringServiceConfig,
    },
    network_id::{NetworkId, PeerNetworkId},
};
use aptos_network::{
    application::{interface::NetworkClient, metadata::PeerMetadata},
    transport::ConnectionMetadata,
};
use aptos_peer_monitoring_service_types::{
    LatencyPingResponse, NetworkInformationResponse, PeerMonitoringServiceMessage,
    PeerMonitoringServiceRequest, PeerMonitoringServiceResponse, ServerProtocolVersionResponse,
};
use aptos_time_service::{MockTimeService, TimeService, TimeServiceTrait};
use aptos_types::PeerId;
use maplit::hashmap;
use std::{
    collections::HashMap,
    future::Future,
    time::{Duration, Instant},
};
use tokio::{
    runtime::Handle,
    time::{sleep, timeout},
};

// Useful test constants
const PAUSE_FOR_MONITOR_SETUP_SECS: u64 = 1;
const MAX_WAIT_TIME_SECS: u64 = 10;
const SLEEP_DURATION_MS: u64 = 500;

/// Elapses enough time for a latency update to occur
pub async fn elapse_latency_update_interval(node_config: NodeConfig, mock_time: MockTimeService) {
    let latency_monitoring_config = node_config.peer_monitoring_service.latency_monitoring;
    mock_time
        .advance_ms_async(latency_monitoring_config.latency_ping_interval_ms + 1)
        .await;
}

/// Elapses enough time for a network info update to occur
pub async fn elapse_network_info_update_interval(
    node_config: NodeConfig,
    mock_time: MockTimeService,
) {
    let network_monitoring_config = node_config.peer_monitoring_service.network_monitoring;
    mock_time
        .advance_ms_async(network_monitoring_config.network_info_request_interval_ms + 1)
        .await;
}

/// Elapses enough time for the monitoring loop to execute
pub async fn elapse_peer_monitor_interval(node_config: NodeConfig, mock_time: MockTimeService) {
    let peer_monitoring_config = node_config.peer_monitoring_service;
    mock_time
        .advance_ms_async(peer_monitoring_config.peer_monitor_interval_ms + 1)
        .await;
}

/// Returns a config where latency pings don't refresh
pub fn get_config_without_latency_pings() -> NodeConfig {
    NodeConfig {
        peer_monitoring_service: PeerMonitoringServiceConfig {
            latency_monitoring: LatencyMonitoringConfig {
                latency_ping_interval_ms: 1_000_000_000, // Unrealistically high
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Returns a config where network info requests don't refresh
pub fn get_config_without_network_info_requests() -> NodeConfig {
    NodeConfig {
        peer_monitoring_service: PeerMonitoringServiceConfig {
            network_monitoring: NetworkMonitoringConfig {
                network_info_request_interval_ms: 1_000_000_000, // Unrealistically high
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Initializes all the peer states by running the peer monitor loop
/// once and ensuring the correct requests and responses are received.
pub async fn initialize_and_verify_peer_states(
    network_id: &NetworkId,
    mock_monitoring_server: &mut MockMonitoringServer,
    peer_monitor_state: &PeerMonitorState,
    node_config: &NodeConfig,
    validator_peer: &PeerNetworkId,
    mock_time: &MockTimeService,
) -> (HashMap<PeerNetworkId, PeerMetadata>, u64) {
    // Elapse enough time for the peer monitor to execute
    let time_before_update = mock_time.now();
    elapse_peer_monitor_interval(node_config.clone(), mock_time.clone()).await;

    // Create the test response data
    let connected_peers_and_metadata = hashmap! { PeerNetworkId::random() => PeerMetadata::new(ConnectionMetadata::mock(PeerId::random())) };
    let distance_from_validators = match network_id {
        NetworkId::Validator => 0,
        NetworkId::Vfn => 1,
        NetworkId::Public => 2,
    };

    // Verify the initial client requests and send responses
    verify_all_requests_and_respond(
        network_id,
        mock_monitoring_server,
        &connected_peers_and_metadata,
        distance_from_validators,
    )
    .await;

    // Wait until the peer state is updated by the client
    wait_for_peer_state_update(
        time_before_update,
        peer_monitor_state,
        validator_peer,
        PeerStateKey::get_all_keys(),
    )
    .await;

    // Verify the new state of the peer monitor
    verify_peer_monitor_state(
        peer_monitor_state,
        validator_peer,
        &connected_peers_and_metadata,
        distance_from_validators,
        1,
    );

    (connected_peers_and_metadata, distance_from_validators)
}

/// Spawns the given task with a timeout
pub async fn spawn_with_timeout(task: impl Future<Output = ()>, timeout_error_message: &str) {
    let timeout_duration = Duration::from_secs(MAX_WAIT_TIME_SECS);
    timeout(timeout_duration, task)
        .await
        .expect(timeout_error_message)
}

/// Spawns the peer monitor
pub async fn start_peer_monitor(
    peer_monitoring_client: PeerMonitoringServiceClient<
        NetworkClient<PeerMonitoringServiceMessage>,
    >,
    peer_monitor_state: &PeerMonitorState,
    time_service: &TimeService,
    node_config: &NodeConfig,
) {
    // Spawn the peer monitor state
    tokio::spawn(start_peer_monitor_with_state(
        node_config.clone(),
        peer_monitoring_client,
        peer_monitor_state.clone(),
        time_service.clone(),
        Some(Handle::current()),
    ));

    // Wait for some time so that the peer monitor starts before we return
    sleep(Duration::from_secs(PAUSE_FOR_MONITOR_SETUP_SECS)).await
}

/// Elapses enough time for a latency ping and handles the ping
pub async fn verify_and_handle_latency_ping(
    network_id: &NetworkId,
    mock_monitoring_server: &mut MockMonitoringServer,
    peer_monitor_state: &PeerMonitorState,
    node_config: &NodeConfig,
    peer_network_id: &PeerNetworkId,
    mock_time: &MockTimeService,
    expected_latency_ping_counter: u64,
    expected_num_recorded_latency_pings: u64,
) {
    // Elapse enough time for a latency ping update
    let time_before_update = mock_time.now();
    elapse_latency_update_interval(node_config.clone(), mock_time.clone()).await;

    // Verify that a single latency request is received and respond
    verify_latency_request_and_respond(
        network_id,
        mock_monitoring_server,
        expected_latency_ping_counter,
        false,
        false,
        false,
    )
    .await;

    // Wait until the latency peer state is updated by the client
    wait_for_peer_state_update(
        time_before_update,
        peer_monitor_state,
        peer_network_id,
        vec![PeerStateKey::LatencyInfo],
    )
    .await;

    // Verify the latency ping state
    verify_peer_latency_state(
        peer_monitor_state,
        peer_network_id,
        expected_num_recorded_latency_pings,
        0,
    );
}

/// Elapses enough time for a network info request and handles the response
pub async fn verify_and_handle_network_info_request(
    network_id: &NetworkId,
    mock_monitoring_server: &mut MockMonitoringServer,
    peer_monitor_state: &PeerMonitorState,
    node_config: &NodeConfig,
    peer_network_id: &PeerNetworkId,
    mock_time: &MockTimeService,
    connected_peers_and_metadata: &HashMap<PeerNetworkId, PeerMetadata>,
    distance_from_validators: u64,
) {
    // Elapse enough time for a network info update
    let time_before_update = mock_time.now();
    elapse_network_info_update_interval(node_config.clone(), mock_time.clone()).await;

    // Verify that a single network info request is received and respond
    verify_network_info_request_and_respond(
        network_id,
        mock_monitoring_server,
        connected_peers_and_metadata,
        distance_from_validators,
        false,
        false,
        false,
    )
    .await;

    // Wait until the network info state is updated by the client
    wait_for_peer_state_update(
        time_before_update,
        peer_monitor_state,
        peer_network_id,
        vec![PeerStateKey::NetworkInfo],
    )
    .await;

    // Verify the network info state
    verify_peer_network_state(
        peer_monitor_state,
        peer_network_id,
        connected_peers_and_metadata,
        distance_from_validators,
        0,
    );
}

/// Verifies that all request types are received by the server
/// and responds to them using the specified data.
pub async fn verify_all_requests_and_respond(
    network_id: &NetworkId,
    mock_monitoring_server: &mut MockMonitoringServer,
    connected_peers_and_metadata: &HashMap<PeerNetworkId, PeerMetadata>,
    distance_from_validators: u64,
) {
    // Create a task that waits for all the requests and sends responses
    let handle_requests = async move {
        // Counters to ensure we only receive one type of each request
        let mut num_received_latency_pings = 0;
        let mut num_received_network_requests = 0;

        // We expect a request to be sent for each peer state type
        let num_state_types = PeerStateKey::get_all_keys().len();
        for _ in 0..num_state_types {
            // Process the peer monitoring request
            let network_request = mock_monitoring_server
                .next_request(network_id)
                .await
                .unwrap();
            let response = match network_request.peer_monitoring_service_request {
                PeerMonitoringServiceRequest::GetNetworkInformation => {
                    // Increment the counter
                    num_received_network_requests += 1;

                    // Return the response
                    PeerMonitoringServiceResponse::NetworkInformation(NetworkInformationResponse {
                        connected_peers_and_metadata: connected_peers_and_metadata.clone(),
                        distance_from_validators,
                    })
                },
                PeerMonitoringServiceRequest::LatencyPing(latency_ping) => {
                    // Increment the counter
                    num_received_latency_pings += 1;

                    // Return the response
                    PeerMonitoringServiceResponse::LatencyPing(LatencyPingResponse {
                        ping_counter: latency_ping.ping_counter,
                    })
                },
                request => panic!("Unexpected monitoring request received: {:?}", request),
            };

            // Send the response
            network_request.response_sender.send(Ok(response));
        }

        // Verify each request was received exactly once
        if (num_received_latency_pings != 1) || (num_received_network_requests != 1) {
            panic!("The requests were not received exactly once!");
        }
    };

    // Spawn the task with a timeout
    spawn_with_timeout(
        handle_requests,
        "Timed-out while waiting for all the requests!",
    )
    .await;
}

/// Verifies that the peer states is empty
pub fn verify_empty_peer_states(peer_monitor_state: &PeerMonitorState) {
    assert!(peer_monitor_state.peer_states.read().is_empty());
}

/// Verifies that a latency ping request is received and sends a
/// response based on the given parameters.
pub async fn verify_latency_request_and_respond(
    network_id: &NetworkId,
    mock_monitoring_server: &mut MockMonitoringServer,
    expected_ping_counter: u64,
    respond_with_invalid_counter: bool,
    respond_with_invalid_message: bool,
    skip_sending_a_response: bool,
) {
    // Create a task that waits for the request and sends a response
    let handle_request = async move {
        // Process the latency ping request
        let network_request = mock_monitoring_server
            .next_request(network_id)
            .await
            .unwrap();
        let response = match network_request.peer_monitoring_service_request {
            PeerMonitoringServiceRequest::LatencyPing(latency_ping) => {
                // Verify the ping counter
                assert_eq!(latency_ping.ping_counter, expected_ping_counter);

                // Create the response
                if respond_with_invalid_counter {
                    // Respond with an invalid ping counter
                    PeerMonitoringServiceResponse::LatencyPing(LatencyPingResponse {
                        ping_counter: 1010101,
                    })
                } else if respond_with_invalid_message {
                    // Respond with the wrong message type
                    PeerMonitoringServiceResponse::ServerProtocolVersion(
                        ServerProtocolVersionResponse { version: 999 },
                    )
                } else {
                    // Send a valid response
                    PeerMonitoringServiceResponse::LatencyPing(LatencyPingResponse {
                        ping_counter: latency_ping.ping_counter,
                    })
                }
            },
            request => panic!("Unexpected monitoring request received: {:?}", request),
        };

        // Send the response
        if !skip_sending_a_response {
            network_request.response_sender.send(Ok(response));
        }
    };

    // Spawn the task with a timeout
    spawn_with_timeout(
        handle_request,
        "Timed-out while waiting for a latency ping request",
    )
    .await;
}

/// Verifies that a network info request is received by the
/// server and sends a response.
pub async fn verify_network_info_request_and_respond(
    network_id: &NetworkId,
    mock_monitoring_server: &mut MockMonitoringServer,
    connected_peers_and_metadata: &HashMap<PeerNetworkId, PeerMetadata>,
    distance_from_validators: u64,
    respond_with_invalid_distance: bool,
    respond_with_invalid_message: bool,
    skip_sending_a_response: bool,
) {
    // Create a task that waits for the request and sends a response
    let handle_request = async move {
        // Process the latency ping request
        let network_request = mock_monitoring_server
            .next_request(network_id)
            .await
            .unwrap();
        let response = match network_request.peer_monitoring_service_request {
            PeerMonitoringServiceRequest::GetNetworkInformation => {
                if respond_with_invalid_distance {
                    // Respond with an invalid distance
                    PeerMonitoringServiceResponse::NetworkInformation(NetworkInformationResponse {
                        connected_peers_and_metadata: connected_peers_and_metadata.clone(),
                        distance_from_validators: 1,
                    })
                } else if respond_with_invalid_message {
                    // Respond with the wrong message type
                    PeerMonitoringServiceResponse::LatencyPing(LatencyPingResponse {
                        ping_counter: 10,
                    })
                } else {
                    // Send a valid response
                    PeerMonitoringServiceResponse::NetworkInformation(NetworkInformationResponse {
                        connected_peers_and_metadata: connected_peers_and_metadata.clone(),
                        distance_from_validators,
                    })
                }
            },
            request => panic!("Unexpected monitoring request received: {:?}", request),
        };

        // Send the response
        if !skip_sending_a_response {
            network_request.response_sender.send(Ok(response));
        }
    };

    // Spawn the task with a timeout
    spawn_with_timeout(
        handle_request,
        "Timed-out while waiting for a network info request",
    )
    .await;
}

/// Verifies the latency state of the peer monitor
pub fn verify_peer_latency_state(
    peer_monitor_state: &PeerMonitorState,
    peer_network_id: &PeerNetworkId,
    expected_num_recorded_latency_pings: u64,
    expected_num_consecutive_failures: u64,
) {
    // Fetch the peer monitoring metadata
    let peer_states = peer_monitor_state.peer_states.read();
    let peer_state = peer_states.get(peer_network_id).unwrap();

    // Verify the latency ping state
    let latency_info_state = peer_state.get_latency_info_state().unwrap();
    assert_eq!(
        latency_info_state.get_recorded_latency_pings().len(),
        expected_num_recorded_latency_pings as usize
    );
    assert_eq!(
        latency_info_state
            .get_request_tracker()
            .read()
            .get_num_consecutive_failures(),
        expected_num_consecutive_failures
    );
}

/// Verifies the state of the peer monitor
pub fn verify_peer_monitor_state(
    peer_monitor_state: &PeerMonitorState,
    peer_network_id: &PeerNetworkId,
    expected_connected_peers_and_metadata: &HashMap<PeerNetworkId, PeerMetadata>,
    expected_distance_from_validators: u64,
    expected_num_recorded_latency_pings: u64,
) {
    // Verify the latency ping state
    verify_peer_latency_state(
        peer_monitor_state,
        peer_network_id,
        expected_num_recorded_latency_pings,
        0,
    );

    // Verify the network state
    verify_peer_network_state(
        peer_monitor_state,
        peer_network_id,
        expected_connected_peers_and_metadata,
        expected_distance_from_validators,
        0,
    );
}

/// Verifies the network state of the peer monitor
pub fn verify_peer_network_state(
    peer_monitor_state: &PeerMonitorState,
    peer_network_id: &PeerNetworkId,
    expected_connected_peers_and_metadata: &HashMap<PeerNetworkId, PeerMetadata>,
    expected_distance_from_validators: u64,
    expected_num_consecutive_failures: u64,
) {
    // Fetch the peer monitoring metadata
    let peer_states = peer_monitor_state.peer_states.read();
    let peer_state = peer_states.get(peer_network_id).unwrap();

    // Verify the network state
    let network_info_state = peer_state.get_network_info_state().unwrap();
    let latest_network_info_response = network_info_state
        .get_latest_network_info_response()
        .unwrap();
    assert_eq!(
        latest_network_info_response.connected_peers_and_metadata,
        expected_connected_peers_and_metadata.clone()
    );
    assert_eq!(
        latest_network_info_response.distance_from_validators,
        expected_distance_from_validators
    );
    assert_eq!(
        network_info_state
            .get_request_tracker()
            .read()
            .get_num_consecutive_failures(),
        expected_num_consecutive_failures
    );
}

/// Waits for the peer monitor state to be updated with
/// a latency ping failure.
pub async fn wait_for_latency_ping_failure(
    peer_monitor_state: &PeerMonitorState,
    peer_network_id: &PeerNetworkId,
    num_expected_consecutive_failures: u64,
) {
    // Create a task that waits for the updated states
    let wait_for_update = async move {
        loop {
            // Fetch the request tracker for the latency state
            let peers_states_lock = peer_monitor_state.peer_states.read();
            let peer_state = peers_states_lock.get(peer_network_id).unwrap();
            let request_tracker = peer_state
                .get_request_tracker(&PeerStateKey::LatencyInfo)
                .unwrap();
            drop(peers_states_lock);

            // Check if the request tracker failures matches the expected number
            let num_consecutive_failures = request_tracker.read().get_num_consecutive_failures();
            if num_consecutive_failures == num_expected_consecutive_failures {
                return; // The peer state was updated!
            }

            // Sleep for some time before retrying
            sleep(Duration::from_millis(SLEEP_DURATION_MS)).await;
        }
    };

    // Spawn the task with a timeout
    spawn_with_timeout(
        wait_for_update,
        "Timed-out while waiting for a latency ping failure!",
    )
    .await;
}

/// Waits for the peer monitor state to be updated with
/// a network info request failure.
pub async fn wait_for_network_info_request_failure(
    peer_monitor_state: &PeerMonitorState,
    peer_network_id: &PeerNetworkId,
    num_expected_consecutive_failures: u64,
) {
    // Create a task that waits for the updated states
    let wait_for_update = async move {
        loop {
            // Fetch the request tracker for the network info state
            let peers_states_lock = peer_monitor_state.peer_states.read();
            let peer_state = peers_states_lock.get(peer_network_id).unwrap();
            let request_tracker = peer_state
                .get_request_tracker(&PeerStateKey::NetworkInfo)
                .unwrap();
            drop(peers_states_lock);

            // Check if the request tracker failures matches the expected number
            let num_consecutive_failures = request_tracker.read().get_num_consecutive_failures();
            if num_consecutive_failures == num_expected_consecutive_failures {
                return; // The peer state was updated!
            }

            // Sleep for some time before retrying
            sleep(Duration::from_millis(SLEEP_DURATION_MS)).await;
        }
    };

    // Spawn the task with a timeout
    spawn_with_timeout(
        wait_for_update,
        "Timed-out while waiting for a network info failure!",
    )
    .await;
}

/// Waits for the peer monitor state to be updated with
/// metadata after the given timestamp.
pub async fn wait_for_peer_state_update(
    time_before_update: Instant,
    peer_monitor_state: &PeerMonitorState,
    peer_network_id: &PeerNetworkId,
    peer_state_keys: Vec<PeerStateKey>,
) {
    // Create a task that waits for the updated states
    let wait_for_update = async move {
        // Go through all peer states and ensure each one is updated
        for peer_state_key in peer_state_keys {
            loop {
                // Fetch the request tracker for the peer state
                let peers_states_lock = peer_monitor_state.peer_states.read();
                let peer_state = peers_states_lock.get(peer_network_id).unwrap();
                let request_tracker = peer_state.get_request_tracker(&peer_state_key).unwrap();
                drop(peers_states_lock);

                // Check if the request tracker has a response with a newer timestamp
                if let Some(last_response_time) = request_tracker.read().get_last_response_time() {
                    if last_response_time > time_before_update {
                        break; // The peer state was updated!
                    }
                };

                // Sleep for some time before retrying
                sleep(Duration::from_millis(SLEEP_DURATION_MS)).await;
            }
        }
    };

    // Spawn the task with a timeout
    spawn_with_timeout(
        wait_for_update,
        "Timed-out while waiting for a peer state update!",
    )
    .await;
}
