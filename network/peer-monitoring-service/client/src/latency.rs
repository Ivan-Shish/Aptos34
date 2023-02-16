use crate::{
    client::PeerMonitoringServiceClient, metrics, Error, LogEntry, LogEvent, LogSchema,
    PeerMonitorState,
};
use aptos_config::{
    config::{LatencyMonitoringConfig, PeerMonitoringServiceConfig},
    network_id::PeerNetworkId,
};
use aptos_id_generator::{IdGenerator, U64IdGenerator};
use aptos_infallible::RwLock;
use aptos_logger::warn;
use aptos_network::application::interface::NetworkClient;
use aptos_peer_monitoring_service_types::{
    LatencyPingRequest, LatencyPingResponse, PeerMonitoringServiceMessage,
    PeerMonitoringServiceRequest,
};
use aptos_time_service::{TimeService, TimeServiceTrait};
use std::{
    collections::{BTreeMap, HashMap},
    ops::Add,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{runtime::Handle, task::JoinHandle};

/// A simple container that holds a single peer's latency ping state
#[derive(Clone, Debug)]
pub struct LatencyPingState {
    in_flight_latency_ping: bool, // If there is a latency ping currently in-flight
    last_latency_ping_time: Option<Instant>, // The most recent latency ping timestamp
    latency_monitoring_config: LatencyMonitoringConfig, // The config for the latency monitoring component
    latency_ping_counter: u64, // The monotonically increasing counter for each ping
    num_consecutive_latency_ping_failures: u64, // The num of consecutive latency ping failures
    recorded_latency_ping_durations_secs: BTreeMap<u64, f64>, // All recorded latency ping durations by counter (secs)
    time_service: TimeService, // The time service to use for duration calculation
}

impl LatencyPingState {
    pub fn new(
        latency_monitoring_config: LatencyMonitoringConfig,
        time_service: TimeService,
    ) -> Self {
        Self {
            in_flight_latency_ping: false,
            last_latency_ping_time: None,
            latency_monitoring_config,
            latency_ping_counter: 0,
            num_consecutive_latency_ping_failures: 0,
            recorded_latency_ping_durations_secs: BTreeMap::new(),
            time_service,
        }
    }

    /// Returns true iff there is a latency ping currently
    /// in-flight for the specified peer.
    pub fn in_flight_latency_ping(&self) -> bool {
        self.in_flight_latency_ping
    }

    /// Marks the peers as having an in-flight latency ping
    pub fn in_flight_latency_ping_started(&mut self) {
        self.in_flight_latency_ping = true
    }

    /// Marks the peers as having completed an in-flight latency ping
    pub fn in_flight_latency_ping_completed(&mut self) {
        self.in_flight_latency_ping = false;
    }

    /// Returns the current latency ping counter and increments it internally
    pub fn get_and_increment_latency_ping_counter(&mut self) -> u64 {
        let latency_ping_counter = self.latency_ping_counter;
        self.latency_ping_counter += 1;
        latency_ping_counter
    }

    /// Returns true iff the peer needs to have a new
    /// latency ping sent (based on the latest ping time).
    pub fn needs_new_latency_ping(&self) -> bool {
        match self.last_latency_ping_time {
            Some(last_latency_ping) => {
                self.time_service.now()
                    > last_latency_ping.add(Duration::from_millis(
                        self.latency_monitoring_config.latency_ping_interval_ms,
                    ))
            },
            None => true, // We need to ping the peer immediately
        }
    }

    /// Records the new latency ping entry for the peer and resets the
    /// consecutive failure counter.
    pub fn record_new_latency_and_reset_failures(
        &mut self,
        latency_ping_counter: u64,
        latency_ping_time_secs: f64,
    ) {
        // Update the last latency ping
        self.last_latency_ping_time = Some(self.time_service.now());

        // Save the latency ping time
        self.recorded_latency_ping_durations_secs
            .insert(latency_ping_counter, latency_ping_time_secs);

        // Perform garbage collection on the recorded latency pings
        let max_num_latency_pings_to_retain = self
            .latency_monitoring_config
            .max_num_latency_pings_to_retain;
        if self.recorded_latency_ping_durations_secs.len() > max_num_latency_pings_to_retain {
            // We only need to pop a single element because insertion only happens in this method.
            // Thus, the size can only ever grow to be 1 greater than the max.
            let _ = self.recorded_latency_ping_durations_secs.pop_first();
        }

        // Reset the number of consecutive ping failures
        self.num_consecutive_latency_ping_failures = 0;
    }

    /// Returns the average latency ping in seconds. If no latency
    /// pings have been recorded, None is returned.
    pub fn get_average_latency_ping_secs(&self) -> Option<f64> {
        let num_latency_pings = self.recorded_latency_ping_durations_secs.len();
        if num_latency_pings > 0 {
            let average_latency_secs_sum: f64 =
                self.recorded_latency_ping_durations_secs.values().sum();
            Some(average_latency_secs_sum / num_latency_pings as f64)
        } else {
            None
        }
    }

    /// Records a new ping failure for the peer
    pub fn record_ping_failure(&mut self) {
        self.num_consecutive_latency_ping_failures += 1;
    }

    /// Returns true iff the number of ping failures is beyond the max failures tolerated
    pub fn ping_failures_beyond_max(&mut self) -> bool {
        self.num_consecutive_latency_ping_failures
            >= self.latency_monitoring_config.max_latency_ping_failures
    }
}

/// Sends a latency ping to the peers that need pinging
pub fn send_peer_latency_pings(
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
    let mut num_in_flight_pings = 0;
    for peer_network_id in connected_peers {
        let should_ping_peer = match peer_monitor_state
            .latency_ping_states
            .read()
            .get(&peer_network_id)
        {
            Some(latency_ping_state) => {
                // If there's an in-flight ping, the peer doesn't need to be pinged.
                // Otherwise, check if enough time has elapsed.
                if latency_ping_state.in_flight_latency_ping() {
                    num_in_flight_pings += 1;
                    false
                } else {
                    latency_ping_state.needs_new_latency_ping()
                }
            },
            None => true, // We should immediately ping the peer
        };

        // Only ping the peer if it needs to be pinged
        if should_ping_peer {
            if let Err(error) = ping_peer(
                peer_monitoring_config.latency_monitoring.clone(),
                peer_monitoring_client.clone(),
                peer_network_id,
                peer_monitor_state.request_id_generator.clone(),
                peer_monitor_state.latency_ping_states.clone(),
                time_service.clone(),
                runtime.clone(),
            ) {
                warn!(LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::UnexpectedErrorEncountered)
                    .peer(&peer_network_id)
                    .error(&error));
            }
        }
    }

    // Update the in-flight latency metrics
    update_in_flight_metrics(num_in_flight_pings);
}

/// Spawns a dedicated task that pings the given peer.
fn ping_peer(
    latency_monitoring_config: LatencyMonitoringConfig,
    peer_monitoring_client: PeerMonitoringServiceClient<
        NetworkClient<PeerMonitoringServiceMessage>,
    >,
    peer_network_id: PeerNetworkId,
    request_id_generator: Arc<U64IdGenerator>,
    latency_ping_states: Arc<RwLock<HashMap<PeerNetworkId, LatencyPingState>>>,
    time_service: TimeService,
    runtime: Option<Handle>,
) -> Result<JoinHandle<()>, Error> {
    // If this is the first time the peer is being pinged, create a new state
    let state_exists = latency_ping_states.read().contains_key(&peer_network_id);
    if !state_exists {
        let latency_ping_state =
            LatencyPingState::new(latency_monitoring_config.clone(), time_service.clone());
        latency_ping_states
            .write()
            .insert(peer_network_id, latency_ping_state);
    }

    // Create the latency ping request and mark the ping as having started
    let latency_ping_request = match latency_ping_states.write().get_mut(&peer_network_id) {
        Some(latency_ping_state) => {
            // Mark the ping as having started. We do this here to prevent
            // the monitor loop from selecting the same peer concurrently.
            latency_ping_state.in_flight_latency_ping_started();

            // Create and return the ping message to send
            let ping_counter = latency_ping_state.get_and_increment_latency_ping_counter();
            LatencyPingRequest { ping_counter }
        },
        None => return Err(missing_peer_error(peer_network_id)),
    };

    // Create the latency ping task for the peer
    let latency_ping = async move {
        // Create the peer monitoring message
        let request_timeout = latency_monitoring_config.max_latency_ping_timeout_ms;
        let latency_ping_request_message =
            PeerMonitoringServiceRequest::LatencyPing(latency_ping_request);

        // Start the ping timer
        let start_time = time_service.now();

        // Send the request to the peer and wait for a response
        let request_id = request_id_generator.next();
        let latency_ping_response: Result<LatencyPingResponse, Error> =
            crate::client::send_request_to_peer_and_decode(
                peer_monitoring_client,
                peer_network_id,
                request_id,
                latency_ping_request_message,
                request_timeout,
            )
            .await;

        // Stop the timer and calculate the duration
        let finish_time = time_service.now();
        let request_duration: Duration = finish_time.duration_since(start_time);
        let request_duration_secs = request_duration.as_secs_f64();

        // Mark the in-flight poll as now complete
        match latency_ping_states.write().get_mut(&peer_network_id) {
            Some(latency_ping_state) => latency_ping_state.in_flight_latency_ping_completed(),
            None => {
                let error = missing_peer_error(peer_network_id);
                warn!(LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::UnexpectedErrorEncountered)
                    .peer(&peer_network_id)
                    .error(&error));
                return;
            },
        }

        // Handle the response type
        let latency_ping_response = match latency_ping_response {
            Ok(latency_ping_response) => latency_ping_response,
            Err(error) => {
                warn!(LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::PeerPingError)
                    .message("Error encountered when pinging peer!")
                    .peer(&peer_network_id)
                    .error(&error));
                handle_ping_failure(latency_ping_states.clone(), peer_network_id);
                return;
            },
        };

        // Verify the latency ping response contains the correct counter
        let request_ping_counter = latency_ping_request.ping_counter;
        let response_ping_counter = latency_ping_response.ping_counter;
        if request_ping_counter != response_ping_counter {
            warn!(LogSchema::new(LogEntry::LatencyPing)
                .event(LogEvent::PeerPingError)
                .peer(&peer_network_id)
                .message(&format!(
                    "Peer responded with the incorrect ping counter! Expected: {:?}, found: {:?}",
                    request_ping_counter, response_ping_counter
                )));
            handle_ping_failure(latency_ping_states.clone(), peer_network_id);
            return;
        }

        // Store the new latency ping result
        match latency_ping_states.write().get_mut(&peer_network_id) {
            Some(latency_ping_state) => latency_ping_state
                .record_new_latency_and_reset_failures(request_ping_counter, request_duration_secs),
            None => {
                let error = missing_peer_error(peer_network_id);
                warn!(LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::UnexpectedErrorEncountered)
                    .peer(&peer_network_id)
                    .error(&error));
            },
        };

        // Update the latency ping metrics
        metrics::observe_value(
            &metrics::REQUEST_LATENCIES,
            latency_ping_request_message.get_label(),
            peer_network_id,
            request_duration_secs,
        );
    };

    // Spawn the latency ping task
    let join_handle = if let Some(runtime) = runtime {
        runtime.spawn(latency_ping)
    } else {
        tokio::spawn(latency_ping)
    };

    Ok(join_handle)
}

/// Handles a ping failure for the specified peer
fn handle_ping_failure(
    latency_ping_states: Arc<RwLock<HashMap<PeerNetworkId, LatencyPingState>>>,
    peer_network_id: PeerNetworkId,
) {
    match latency_ping_states.write().get_mut(&peer_network_id) {
        Some(latency_ping_state) => {
            // Update the number of ping failures
            latency_ping_state.record_ping_failure();

            // TODO: If the number of ping failures is too high, disconnect from the node
            if latency_ping_state.ping_failures_beyond_max() {
                warn!(LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::TooManyPingFailures)
                    .peer(&peer_network_id)
                    .message("Too many ping failures occurred for the peer!"));
            }
        },
        None => {
            let error = missing_peer_error(peer_network_id);
            warn!(LogSchema::new(LogEntry::LatencyPing)
                .event(LogEvent::UnexpectedErrorEncountered)
                .peer(&peer_network_id)
                .error(&error));
        },
    };
}

/// A simple helper to return an error for a missing peer
fn missing_peer_error(peer_network_id: PeerNetworkId) -> Error {
    Error::UnexpectedError(format!(
        "Failed to find the peer. This shouldn't happen! Peer: {:?}",
        peer_network_id
    ))
}

/// Updates the in-flight metrics for on-going latency pings
fn update_in_flight_metrics(num_in_flight_pings: u64) {
    // TODO: Avoid having to use a dummy request to get the label
    let latency_ping_label =
        PeerMonitoringServiceRequest::LatencyPing(LatencyPingRequest { ping_counter: 0 })
            .get_label();

    metrics::update_in_flight_pings(latency_ping_label, num_in_flight_pings);
}
