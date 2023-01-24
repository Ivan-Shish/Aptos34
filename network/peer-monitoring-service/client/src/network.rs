use crate::{
    logging::{LogEntry, LogEvent, LogSchema},
    metrics, Error,
};
use aptos_config::network_id::PeerNetworkId;
use aptos_logger::{trace, warn};
use aptos_network::application::{
    interface::{NetworkClient, NetworkClientInterface},
    storage::PeersAndMetadata,
};
use aptos_peer_monitoring_service_types::{
    PeerMonitoringServiceMessage, PeerMonitoringServiceRequest, PeerMonitoringServiceResponse,
    UnexpectedResponseError,
};
use std::{sync::Arc, time::Duration};

/// The interface for sending peer monitoring service requests and querying
/// peer information.
#[derive(Clone, Debug)]
pub struct PeerMonitoringServiceClient<NetworkClient> {
    network_client: NetworkClient,
}

impl<NetworkClient: NetworkClientInterface<PeerMonitoringServiceMessage>>
    PeerMonitoringServiceClient<NetworkClient>
{
    pub fn new(network_client: NetworkClient) -> Self {
        Self { network_client }
    }

    pub async fn send_request(
        &self,
        recipient: PeerNetworkId,
        request: PeerMonitoringServiceRequest,
        timeout: Duration,
    ) -> Result<PeerMonitoringServiceResponse, Error> {
        let response = self
            .network_client
            .send_to_peer_rpc(
                PeerMonitoringServiceMessage::Request(request),
                timeout,
                recipient,
            )
            .await
            .map_err(|error| Error::NetworkError(error.to_string()))?;
        match response {
            PeerMonitoringServiceMessage::Response(Ok(response)) => Ok(response),
            PeerMonitoringServiceMessage::Response(Err(err)) => {
                Err(Error::PeerMonitoringServiceError(err))
            },
            PeerMonitoringServiceMessage::Request(request) => Err(Error::NetworkError(format!(
                "Got peer monitoring request instead of response! Request: {:?}",
                request
            ))),
        }
    }

    pub fn get_peers_and_metadata(&self) -> Arc<PeersAndMetadata> {
        self.network_client.get_peers_and_metadata()
    }
}

/// Sends a request to a specific peer and decodes the response
pub async fn send_request_to_peer_and_decode<T>(
    peer_monitoring_client: PeerMonitoringServiceClient<
        NetworkClient<PeerMonitoringServiceMessage>,
    >,
    peer: PeerNetworkId,
    request_id: u64,
    request: PeerMonitoringServiceRequest,
    request_timeout_ms: u64,
) -> Result<T, Error>
where
    T: TryFrom<PeerMonitoringServiceResponse, Error = UnexpectedResponseError>,
{
    // Send the request and get the response
    let response = send_request_to_peer(
        peer_monitoring_client,
        peer,
        request_id,
        request,
        request_timeout_ms,
    )
    .await?;

    // Try to convert the response enum into the exact variant we're expecting
    Ok(T::try_from(response)?)
}

/// Sends a request to a specific peer with the given ID and timeout
async fn send_request_to_peer(
    peer_monitoring_client: PeerMonitoringServiceClient<
        NetworkClient<PeerMonitoringServiceMessage>,
    >,
    peer_network_id: PeerNetworkId,
    request_id: u64,
    request: PeerMonitoringServiceRequest,
    request_timeout_ms: u64,
) -> Result<PeerMonitoringServiceResponse, Error> {
    trace!(
        (LogSchema::new(LogEntry::LatencyPing)
            .event(LogEvent::SendRequest)
            .request_type(request.get_label())
            .request_id(request_id)
            .peer(&peer_network_id)
            .request_data(&request))
    );
    metrics::increment_request_counter(
        &metrics::SENT_REQUESTS,
        request.get_label(),
        peer_network_id,
    );

    // Send the request and process the result
    let result = peer_monitoring_client
        .send_request(
            peer_network_id,
            request,
            Duration::from_millis(request_timeout_ms),
        )
        .await;
    match result {
        Ok(response) => {
            trace!(
                (LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::ResponseSuccess)
                    .request_type(request.get_label())
                    .request_id(request_id)
                    .peer(&peer_network_id))
            );
            metrics::increment_request_counter(
                &metrics::SUCCESS_RESPONSES,
                request.get_label(),
                peer_network_id,
            );
            Ok(response)
        },
        Err(error) => {
            warn!(
                (LogSchema::new(LogEntry::LatencyPing)
                    .event(LogEvent::ResponseError)
                    .request_type(request.get_label())
                    .request_id(request_id)
                    .peer(&peer_network_id)
                    .error(&error))
            );
            metrics::increment_request_counter(
                &metrics::ERROR_RESPONSES,
                error.get_label(),
                peer_network_id,
            );
            Err(error)
        },
    }
}
