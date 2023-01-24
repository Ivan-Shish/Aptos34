use crate::{metrics, network::PeerMonitoringServiceClient, Error, LogEntry, LogEvent, LogSchema};
use aptos_config::{config::LatencyMonitoringConfig, network_id::PeerNetworkId};
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
    last_latency_ping: Option<Instant>, // The most recent latency ping timestamp
    latency_monitoring_config: LatencyMonitoringConfig, // The config for the latency monitoring component
    latency_ping_counter: u64, // The monotonically increasing counter for each ping
    num_consecutive_latency_ping_failures: u64, // The num of consecutive latency ping failures
    recorded_latency_ping_durations: BTreeMap<u64, f64>, // All recorded latency ping durations (by counter)
    time_service: TimeService, // The time service to use for duration calculation
}

impl LatencyPingState {
    pub fn new(
        latency_monitoring_config: LatencyMonitoringConfig,
        time_service: TimeService,
    ) -> Self {
        Self {
            in_flight_latency_ping: false,
            last_latency_ping: None,
            latency_monitoring_config,
            latency_ping_counter: 0,
            num_consecutive_latency_ping_failures: 0,
            recorded_latency_ping_durations: BTreeMap::new(),
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
    pub fn needs_new_latency_ping(&self, latency_ping_interval_ms: u64) -> bool {
        match self.last_latency_ping {
            Some(last_latency_ping) => {
                self.time_service.now()
                    > last_latency_ping.add(Duration::from_millis(latency_ping_interval_ms))
            },
            None => true, // We need to ping the peer immediately
        }
    }

    /// Records the new latency ping entry for the peer
    pub fn record_new_latency(&mut self, latency_ping_counter: u64, latency_ping_time_secs: f64) {
        // Update the last latency ping
        self.last_latency_ping = Some(self.time_service.now());

        // Save the latency ping time
        self.recorded_latency_ping_durations
            .insert(latency_ping_counter, latency_ping_time_secs);

        // Perform garbage collection on the recorded latency pings
        let max_num_latency_pings_to_retain = self
            .latency_monitoring_config
            .max_num_latency_pings_to_retain;
        if self.recorded_latency_ping_durations.len() > max_num_latency_pings_to_retain {
            // We only need to pop a single element because insertion only happens in this method.
            // Thus, the size can only ever grow to be 1 greater than the max.
            let _ = self.recorded_latency_ping_durations.pop_first();
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

/// Spawns a dedicated task that pings the given peer.
pub fn ping_peer(
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
        None => {
            return Err(Error::UnexpectedError(format!(
                "Failed to find the peer. This shouldn't happen! Peer: {:?}",
                peer_network_id
            )))
        },
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
            crate::network::send_request_to_peer_and_decode(
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
                warn!(
                    (LogSchema::new(LogEntry::LatencyPing)
                        .event(LogEvent::UnexpectedErrorEncountered)
                        .peer(&peer_network_id)
                        .message(&format!(
                            "Failed to find the peer. This shouldn't happen! Peer: {:?}",
                            peer_network_id
                        )))
                );
                return;
            },
        }

        // Handle the response type
        let latency_ping_response = match latency_ping_response {
            Ok(latency_ping_response) => latency_ping_response,
            Err(error) => {
                warn!(
                    (LogSchema::new(LogEntry::LatencyPing)
                        .event(LogEvent::PeerPingError)
                        .message("Error encountered when pinging peer!")
                        .error(&error)
                        .peer(&peer_network_id))
                );
                update_ping_failure_counter(latency_ping_states.clone(), peer_network_id);
                return;
            },
        };

        // Verify the latency ping response contains the correct counter
        let request_ping_counter = latency_ping_request.ping_counter;
        let response_ping_counter = latency_ping_response.ping_counter;
        if request_ping_counter != response_ping_counter {
            warn!(
                    (LogSchema::new(LogEntry::LatencyPing)
                        .event(LogEvent::PeerPingError)
                        .message(&format!("Peer responded with the incorrect ping counter! Expected: {:?}, found: {:?}", request_ping_counter, response_ping_counter))
                        .peer(&peer_network_id))
                );
            update_ping_failure_counter(latency_ping_states.clone(), peer_network_id);
            return;
        }

        // Store the new latency ping result
        match latency_ping_states.write().get_mut(&peer_network_id) {
            Some(latency_ping_state) => {
                latency_ping_state.record_new_latency(request_ping_counter, request_duration_secs)
            },
            None => {
                warn!(
                    (LogSchema::new(LogEntry::LatencyPing)
                        .event(LogEvent::UnexpectedErrorEncountered)
                        .peer(&peer_network_id)
                        .message(&format!(
                            "Failed to find the peer. This shouldn't happen! Peer: {:?}",
                            peer_network_id
                        )))
                );
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

/// Updates the ping failure counter for the peer
fn update_ping_failure_counter(
    latency_ping_states: Arc<RwLock<HashMap<PeerNetworkId, LatencyPingState>>>,
    peer_network_id: PeerNetworkId,
) {
    match latency_ping_states.write().get_mut(&peer_network_id) {
        Some(latency_ping_state) => {
            latency_ping_state.record_ping_failure()
        },
        None => {
            warn!(
                (LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::UnexpectedErrorEncountered)
                    .peer(&peer_network_id)
                    .message(&format!(
                        "Failed to find the peer. This shouldn't happen! Peer: {:?}",
                        peer_network_id
                    )))
            );
        },
    };
}
