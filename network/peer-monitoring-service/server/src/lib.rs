// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    logging::{LogEntry, LogSchema},
    metrics::{increment_counter, start_timer},
    network::PeerMonitoringServiceNetworkEvents,
};
use aptos_bounded_executor::BoundedExecutor;
use aptos_config::config::PeerMonitoringServiceConfig;
use aptos_logger::prelude::*;
use aptos_network::{application::storage::PeersAndMetadata, ProtocolId};
use aptos_peer_monitoring_service_types::{
    LatencyPingRequest, LatencyPingResponse, PeerMonitoringServiceError,
    PeerMonitoringServiceRequest, PeerMonitoringServiceResponse, Result,
    ServerProtocolVersionResponse,
};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::runtime::Handle;

mod logging;
pub mod metrics;
pub mod network;

#[cfg(test)]
mod tests;

/// Peer monitoring server constants
pub const PEER_MONITORING_SERVER_VERSION: u64 = 1;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Eq, Serialize)]
pub enum Error {
    #[error("Invalid request received: {0}")]
    InvalidRequest(String),
    #[error("Unexpected error encountered: {0}")]
    UnexpectedErrorEncountered(String),
}

impl Error {
    /// Returns a summary label for the error type
    fn get_label(&self) -> &'static str {
        match self {
            Error::InvalidRequest(_) => "invalid_request",
            Error::UnexpectedErrorEncountered(_) => "unexpected_error",
        }
    }
}

/// The server-side actor for the peer monitoring service
pub struct PeerMonitoringServiceServer {
    bounded_executor: BoundedExecutor,
    network_requests: PeerMonitoringServiceNetworkEvents,
    peers_and_metadata: Arc<PeersAndMetadata>,
}

impl PeerMonitoringServiceServer {
    pub fn new(
        config: PeerMonitoringServiceConfig,
        executor: Handle,
        network_requests: PeerMonitoringServiceNetworkEvents,
        peers_and_metadata: Arc<PeersAndMetadata>,
    ) -> Self {
        let bounded_executor =
            BoundedExecutor::new(config.max_concurrent_requests as usize, executor);

        Self {
            bounded_executor,
            network_requests,
            peers_and_metadata,
        }
    }

    /// Starts the peer monitoring service server thread
    pub async fn start(mut self) {
        // Handle the service requests
        while let Some(network_request) = self.network_requests.next().await {
            // Log the request
            let peer_network_id = network_request.peer_network_id;
            let protocol_id = network_request.protocol_id;
            let peer_monitoring_service_request = network_request.peer_monitoring_service_request;
            let response_sender = network_request.response_sender;
            trace!(LogSchema::new(LogEntry::ReceivedPeerMonitoringRequest)
                .request(&peer_monitoring_service_request)
                .message(&format!(
                    "Received peer monitoring request. Peer: {:?}, protocol: {:?}.",
                    peer_network_id, protocol_id,
                )));

            // All handler methods are currently CPU-bound so we want
            // to spawn on the blocking thread pool.
            let peer_metadata = self.peers_and_metadata.clone();
            self.bounded_executor
                .spawn_blocking(move || {
                    let response = Handler::new(peer_metadata)
                        .call(protocol_id, peer_monitoring_service_request);
                    log_monitoring_service_response(&response);
                    response_sender.send(response);
                })
                .await;
        }
    }
}

/// The `Handler` is the "pure" inbound request handler. It contains all the
/// necessary context and state needed to construct a response to an inbound
/// request. We usually clone/create a new handler for every request.
#[derive(Clone)]
pub struct Handler {
    _peers_and_metadata: Arc<PeersAndMetadata>,
}

impl Handler {
    pub fn new(peers_and_metadata: Arc<PeersAndMetadata>) -> Self {
        Self {
            _peers_and_metadata: peers_and_metadata,
        }
    }

    pub fn call(
        &self,
        protocol: ProtocolId,
        request: PeerMonitoringServiceRequest,
    ) -> Result<PeerMonitoringServiceResponse> {
        // Update the request count
        increment_counter(
            &metrics::PEER_MONITORING_REQUESTS_RECEIVED,
            protocol,
            request.get_label(),
        );

        // Time the request processing (the timer will stop when it's dropped)
        let _timer = start_timer(
            &metrics::PEER_MONITORING_REQUEST_PROCESSING_LATENCY,
            protocol,
            request.get_label(),
        );

        // Process the request
        let response = match &request {
            PeerMonitoringServiceRequest::GetNetworkInformation => self.get_network_information(),
            PeerMonitoringServiceRequest::GetServerProtocolVersion => {
                self.get_server_protocol_version()
            },
            PeerMonitoringServiceRequest::GetSystemInformation => self.get_system_information(),
            PeerMonitoringServiceRequest::LatencyPing(request) => self.handle_latency_ping(request),
        };

        // Process the response and handle any errors
        match response {
            Err(error) => {
                // Log the error and update the counters
                increment_counter(
                    &metrics::PEER_MONITORING_ERRORS_ENCOUNTERED,
                    protocol,
                    error.get_label(),
                );
                error!(LogSchema::new(LogEntry::PeerMonitoringServiceError)
                    .error(&error)
                    .request(&request));

                // Return an appropriate response to the client
                match error {
                    Error::InvalidRequest(error) => {
                        Err(PeerMonitoringServiceError::InvalidRequest(error))
                    },
                    error => Err(PeerMonitoringServiceError::InternalError(error.to_string())),
                }
            },
            Ok(response) => {
                // The request was successful
                increment_counter(
                    &metrics::PEER_MONITORING_RESPONSES_SENT,
                    protocol,
                    response.get_label(),
                );
                Ok(response)
            },
        }
    }

    fn get_network_information(&self) -> Result<PeerMonitoringServiceResponse, Error> {
        Err(Error::InvalidRequest(
            "get_network_information() is currently unsupported!".into(),
        ))
    }

    fn get_server_protocol_version(&self) -> Result<PeerMonitoringServiceResponse, Error> {
        let server_protocol_version_response = ServerProtocolVersionResponse {
            version: PEER_MONITORING_SERVER_VERSION,
        };
        Ok(PeerMonitoringServiceResponse::ServerProtocolVersion(
            server_protocol_version_response,
        ))
    }

    fn get_system_information(&self) -> Result<PeerMonitoringServiceResponse, Error> {
        Err(Error::InvalidRequest(
            "get_system_information() is currently unsupported!".into(),
        ))
    }

    fn handle_latency_ping(
        &self,
        latency_ping_request: &LatencyPingRequest,
    ) -> Result<PeerMonitoringServiceResponse, Error> {
        let latency_ping_response = LatencyPingResponse {
            ping_counter: latency_ping_request.ping_counter,
        };
        Ok(PeerMonitoringServiceResponse::LatencyPing(
            latency_ping_response,
        ))
    }
}

/// Logs the response sent by the monitoring service for a request
fn log_monitoring_service_response(
    monitoring_service_response: &Result<PeerMonitoringServiceResponse, PeerMonitoringServiceError>,
) {
    match monitoring_service_response {
        Ok(response) => {
            let response = format!("{:?}", response);
            debug!(LogSchema::new(LogEntry::SentPeerMonitoringResponse).response(&response));
        },
        Err(error) => {
            let error = format!("{:?}", error);
            debug!(LogSchema::new(LogEntry::SentPeerMonitoringResponse).response(&error));
        },
    };
}
