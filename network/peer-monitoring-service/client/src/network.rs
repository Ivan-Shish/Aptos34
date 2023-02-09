use crate::{
    client::PeerMonitoringServiceClient, metrics, Error, LogEntry, LogEvent, LogSchema,
    PeerMonitorState,
};
use aptos_config::{
    config::{NetworkMonitoringConfig, PeerMonitoringServiceConfig},
    network_id::PeerNetworkId,
};
use aptos_id_generator::{IdGenerator, U64IdGenerator};
use aptos_infallible::RwLock;
use aptos_logger::warn;
use aptos_network::application::interface::NetworkClient;
use aptos_peer_monitoring_service_types::{
    NetworkInformationResponse, PeerMonitoringServiceMessage, PeerMonitoringServiceRequest,
};
use aptos_time_service::{TimeService, TimeServiceTrait};
use std::{
    collections::HashMap,
    ops::Add,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{runtime::Handle, task::JoinHandle};

/// A simple container that holds a single peer's network info state
#[derive(Clone, Debug)]
pub struct NetworkInfoState {
    in_flight_request: bool, // If there is a request currently in-flight
    last_response_time: Option<Instant>, // The most recent network info response timestamp
    network_monitoring_config: NetworkMonitoringConfig, // The config for the latency monitoring component
    recorded_network_info_response: Option<NetworkInformationResponse>, // The last network info response
    time_service: TimeService, // The time service to use for duration calculation
}

impl NetworkInfoState {
    pub fn new(
        network_monitoring_config: NetworkMonitoringConfig,
        time_service: TimeService,
    ) -> Self {
        Self {
            in_flight_request: false,
            last_response_time: None,
            network_monitoring_config,
            recorded_network_info_response: None,
            time_service,
        }
    }

    /// Returns true iff there is a network info request currently
    /// in-flight for the specified peer.
    pub fn in_flight_network_info_request(&self) -> bool {
        self.in_flight_request
    }

    /// Marks the peers as having an in-flight network info request
    pub fn in_flight_request_started(&mut self) {
        self.in_flight_request = true
    }

    /// Marks the peers as having completed an in-flight nertwork info request
    pub fn in_flight_request_completed(&mut self) {
        self.in_flight_request = false;
    }

    /// Returns true iff the peer needs to have a new
    /// network info request sent (based on the latest response time).
    pub fn needs_new_network_info_request(&self) -> bool {
        match self.last_response_time {
            Some(last_response_time) => {
                self.time_service.now()
                    > last_response_time.add(Duration::from_millis(
                        self.network_monitoring_config
                            .network_info_request_interval_ms,
                    ))
            },
            None => true, // We need to send a request to the peer immediately
        }
    }

    /// Records the new network info response for the peer
    pub fn record_network_info_response(
        &mut self,
        network_info_response: NetworkInformationResponse,
    ) {
        // Update the last response time
        self.last_response_time = Some(self.time_service.now());

        // Save the network info
        self.recorded_network_info_response = Some(network_info_response);
    }

    /// Returns the latest network info response
    pub fn get_latest_network_info_response(&self) -> Option<NetworkInformationResponse> {
        self.recorded_network_info_response.clone()
    }
}

/// Sends a latency ping to the peers that need pinging
pub fn send_network_info_requests(
    peer_monitoring_config: PeerMonitoringServiceConfig,
    time_service: TimeService,
    runtime: Option<Handle>,
    peer_monitoring_client: PeerMonitoringServiceClient<
        NetworkClient<PeerMonitoringServiceMessage>,
    >,
    peer_monitor_state: &PeerMonitorState,
    connected_peers: Vec<PeerNetworkId>,
) {
    // Go through all connected peers and ping the ones that need to be refreshed
    let mut num_in_flight_requests = 0;
    for peer_network_id in connected_peers {
        let should_ping_peer = match peer_monitor_state
            .network_info_states
            .read()
            .get(&peer_network_id)
        {
            Some(network_info_state) => {
                // If there's an in-flight request, the peer doesn't need to be pinged.
                // Otherwise, check if enough time has elapsed.
                if network_info_state.in_flight_network_info_request() {
                    num_in_flight_requests += 1;
                    false
                } else {
                    network_info_state.needs_new_network_info_request()
                }
            },
            None => true, // We should immediately ping the peer
        };

        // Only send the request if the state needs to be updated
        if should_ping_peer {
            if let Err(error) = request_network_info(
                peer_monitoring_config.network_monitoring.clone(),
                peer_monitoring_client.clone(),
                peer_network_id,
                peer_monitor_state.request_id_generator.clone(),
                peer_monitor_state.network_info_states.clone(),
                time_service.clone(),
                runtime.clone(),
            ) {
                warn!(LogSchema::new(LogEntry::NetworkInfoRequest)
                    .event(LogEvent::UnexpectedErrorEncountered)
                    .peer(&peer_network_id)
                    .error(&error));
            }
        }
    }

    // Update the in-flight metrics
    update_in_flight_metrics(num_in_flight_requests);
}

/// Spawns a dedicated task that pings the given peer.
fn request_network_info(
    network_monitoring_config: NetworkMonitoringConfig,
    peer_monitoring_client: PeerMonitoringServiceClient<
        NetworkClient<PeerMonitoringServiceMessage>,
    >,
    peer_network_id: PeerNetworkId,
    request_id_generator: Arc<U64IdGenerator>,
    network_info_states: Arc<RwLock<HashMap<PeerNetworkId, NetworkInfoState>>>,
    time_service: TimeService,
    runtime: Option<Handle>,
) -> Result<JoinHandle<()>, Error> {
    // If this is the first time the state is being fetched, create a new state
    let state_exists = network_info_states.read().contains_key(&peer_network_id);
    if !state_exists {
        let network_info_state =
            NetworkInfoState::new(network_monitoring_config.clone(), time_service.clone());
        network_info_states
            .write()
            .insert(peer_network_id, network_info_state);
    }

    // Create the network info request and mark the request as having started
    let network_info_request = match network_info_states.write().get_mut(&peer_network_id) {
        Some(network_info_state) => {
            // Mark the ping as having started. We do this here to prevent
            // the monitor loop from selecting the same peer concurrently.
            network_info_state.in_flight_request_started();

            // Create and return the message to send
            PeerMonitoringServiceRequest::GetNetworkInformation
        },
        None => return Err(missing_peer_error(peer_network_id)),
    };

    // Create the latency ping task for the peer
    let network_info_request = async move {
        // Start the request timer
        let start_time = time_service.now();

        // Send the request to the peer and wait for a response
        let request_id = request_id_generator.next();
        let network_info_response: Result<NetworkInformationResponse, Error> =
            crate::client::send_request_to_peer_and_decode(
                peer_monitoring_client,
                peer_network_id,
                request_id,
                network_info_request,
                network_monitoring_config.max_network_info_request_timeout_ms,
            )
            .await;

        // Stop the timer and calculate the duration
        let finish_time = time_service.now();
        let request_duration: Duration = finish_time.duration_since(start_time);
        let request_duration_secs = request_duration.as_secs_f64();

        // Mark the in-flight request as now complete
        match network_info_states.write().get_mut(&peer_network_id) {
            Some(network_info_state) => network_info_state.in_flight_request_completed(),
            None => {
                let error = missing_peer_error(peer_network_id);
                warn!(LogSchema::new(LogEntry::NetworkInfoRequest)
                    .event(LogEvent::UnexpectedErrorEncountered)
                    .peer(&peer_network_id)
                    .error(&error));
                return;
            },
        }

        // Handle the response type
        let network_info_response = match network_info_response {
            Ok(network_info_response) => network_info_response,
            Err(error) => {
                warn!(LogSchema::new(LogEntry::NetworkInfoRequest)
                    .event(LogEvent::ResponseError)
                    .message("Error encountered when pinging peer!")
                    .peer(&peer_network_id)
                    .error(&error));
                return;
            },
        };

        // Store the new latency ping result
        match network_info_states.write().get_mut(&peer_network_id) {
            Some(network_info_state) => {
                network_info_state.record_network_info_response(network_info_response)
            },
            None => {
                let error = missing_peer_error(peer_network_id);
                warn!(LogSchema::new(LogEntry::NetworkInfoRequest)
                    .event(LogEvent::UnexpectedErrorEncountered)
                    .peer(&peer_network_id)
                    .error(&error));
            },
        };

        // Update the latency ping metrics
        metrics::observe_value(
            &metrics::REQUEST_LATENCIES,
            network_info_request.get_label(),
            peer_network_id,
            request_duration_secs,
        );
    };

    // Spawn the request task
    let join_handle = if let Some(runtime) = runtime {
        runtime.spawn(network_info_request)
    } else {
        tokio::spawn(network_info_request)
    };

    Ok(join_handle)
}

/// A simple helper to return an error for a missing peer
fn missing_peer_error(peer_network_id: PeerNetworkId) -> Error {
    Error::UnexpectedError(format!(
        "Failed to find the peer. This shouldn't happen! Peer: {:?}",
        peer_network_id
    ))
}

/// Updates the in-flight metrics for on-going network info requests
fn update_in_flight_metrics(num_in_flight_pings: u64) {
    // TODO: Avoid having to use a dummy request to get the label
    let request_label = PeerMonitoringServiceRequest::GetNetworkInformation.get_label();
    metrics::update_in_flight_pings(request_label, num_in_flight_pings);
}
