// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, time::Duration};
use thiserror::Error;

pub type Result<T, E = PeerMonitoringServiceError> = ::std::result::Result<T, E>;

/// An error that can be returned to the client on a failure to
/// process a request.
#[derive(Clone, Debug, Deserialize, Error, PartialEq, Eq, Serialize)]
pub enum PeerMonitoringServiceError {
    #[error("Internal service error: {0}")]
    InternalError(String),
    #[error("Invalid service request: {0}")]
    InvalidRequest(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum PeerMonitoringServiceMessage {
    /// A request to the peer monitoring service
    Request(PeerMonitoringServiceRequest),
    /// A response from the peer monitoring service
    Response(Result<PeerMonitoringServiceResponse>),
}

/// A peer monitoring service request
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PeerMonitoringServiceRequest {
    GetNetworkInformation,    // Returns relevant network information for the peer
    GetServerProtocolVersion, // Fetches the protocol version run by the server
    GetSystemInformation,     // Returns relevant system information for the peer
    LatencyPing(LatencyPingRequest), // A simple message used by the client to ensure liveness and measure latency
}

impl PeerMonitoringServiceRequest {
    /// Returns a summary label for the request
    pub fn get_label(&self) -> &'static str {
        match self {
            Self::GetNetworkInformation => "get_network_information",
            Self::GetServerProtocolVersion => "get_server_protocol_version",
            Self::GetSystemInformation => "get_system_information",
            Self::LatencyPing(_) => "latency_ping",
        }
    }
}

/// The latency ping request
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LatencyPingRequest {
    pub ping_counter: u64, // A monotonically increasing counter to verify latency ping responses
}

/// A peer monitoring service response
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum PeerMonitoringServiceResponse {
    LatencyPing(LatencyPingResponse), // A simple message to respond to latency checks (i.e., pings)
    NetworkInformation(NetworkInformationResponse), // Holds the response for network information
    ServerProtocolVersion(ServerProtocolVersionResponse), // Returns the current server protocol version
    SystemInformation(SystemInformationResponse), // Holds the response for system information
}

impl PeerMonitoringServiceResponse {
    /// Returns a summary label for the response
    pub fn get_label(&self) -> &'static str {
        match self {
            Self::LatencyPing(_) => "latency_ping",
            Self::NetworkInformation(_) => "network_information",
            Self::ServerProtocolVersion(_) => "server_protocol_version",
            Self::SystemInformation(_) => "system_information",
        }
    }
}

/// A response for the latency ping request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LatencyPingResponse {
    pub ping_counter: u64, // A monotonically increasing counter to verify latency ping responses
}

/// A response for the network information request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NetworkInformationResponse {
    pub depth_from_validators: u64, // The depth of the peers from the validator set
                                    // TODO: add the rest of the information here!
}

/// A response for the server protocol version request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ServerProtocolVersionResponse {
    pub version: u64, // The version of the peer monitoring service run by the server
}

/// A response for the system information request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SystemInformationResponse {
    pub uptime: Duration, // The uptime of the node
                          // TODO: add the rest of the information here!
}

#[derive(Clone, Debug, Error)]
#[error("Unexpected response variant: {0}")]
pub struct UnexpectedResponseError(pub String);

impl TryFrom<PeerMonitoringServiceResponse> for LatencyPingResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::LatencyPing(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected latency_ping_response, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<PeerMonitoringServiceResponse> for NetworkInformationResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::NetworkInformation(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected network_information_response, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<PeerMonitoringServiceResponse> for ServerProtocolVersionResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::ServerProtocolVersion(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected server_protocol_version_response, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<PeerMonitoringServiceResponse> for SystemInformationResponse {
    type Error = UnexpectedResponseError;

    fn try_from(response: PeerMonitoringServiceResponse) -> Result<Self, Self::Error> {
        match response {
            PeerMonitoringServiceResponse::SystemInformation(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected system_information_response, found {}",
                response.get_label()
            ))),
        }
    }
}
