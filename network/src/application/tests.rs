// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    application::{
        interface::{NetworkClient, NetworkClientInterface, NetworkServiceEvents},
        metadata::ConnectionState,
        storage::PeersAndMetadata,
    },
    peer_manager::{
        ConnectionRequestSender, PeerManagerNotification, PeerManagerRequest,
        PeerManagerRequestSender,
    },
    protocols::{
        network::{Event, NetworkEvents, NetworkSender, NewNetworkEvents, NewNetworkSender},
        rpc::InboundRpcRequest,
        wire::handshake::v1::{ProtocolId, ProtocolIdSet},
    },
    transport::ConnectionMetadata,
};
use aptos_channels::{aptos_channel, message_queues::QueueStyle};
use aptos_config::network_id::{NetworkId, PeerNetworkId};
use aptos_types::PeerId;
use futures::channel::oneshot;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, time::Duration};
use tokio::time::timeout;

// Useful test constants for timeouts
const MAX_MESSAGE_TIMEOUT_SECS: u64 = 2;
const MAX_CHANNEL_TIMEOUT_SECS: u64 = 1;

#[test]
fn test_peers_and_metadata_interface() {
    // Create the peers and metadata container
    let network_ids = [NetworkId::Validator, NetworkId::Vfn];
    let peers_and_metadata = PeersAndMetadata::new(&network_ids);

    // Verify the initial state of storage
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(peers_and_metadata.get_networks().count(), 2);
    for network_id in &network_ids {
        assert!(peers_and_metadata
            .get_networks()
            .any(|id| id == *network_id));
    }

    // Create two peers and connections
    let peer_network_id_1 = PeerNetworkId::new(NetworkId::Validator, PeerId::random());
    let peer_network_id_2 = PeerNetworkId::new(NetworkId::Vfn, PeerId::random());
    let mut connection_1 = ConnectionMetadata::mock(peer_network_id_1.peer_id());
    let mut connection_2 = ConnectionMetadata::mock(peer_network_id_2.peer_id());

    // Update the connections to support different protocol IDs
    let peer_protocols_1 = [ProtocolId::MempoolDirectSend, ProtocolId::StorageServiceRpc];
    let peer_protocols_2 = [ProtocolId::MempoolDirectSend, ProtocolId::ConsensusRpcBcs];
    connection_1.application_protocols = ProtocolIdSet::from_iter(peer_protocols_1);
    connection_2.application_protocols = ProtocolIdSet::from_iter(peer_protocols_2);

    // Insert the two connections into storage and verify the number of connected peers
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_1, connection_1.clone())
        .unwrap();
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_2, connection_2.clone())
        .unwrap();
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        2
    );

    // Verify the supported protocols by protocol type
    assert_eq!(
        peers_and_metadata
            .get_connected_supported_peers(&[ProtocolId::MempoolDirectSend])
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        peers_and_metadata
            .get_connected_supported_peers(&[ProtocolId::StorageServiceRpc])
            .unwrap(),
        vec![peer_network_id_1]
    );
    assert_eq!(
        peers_and_metadata
            .get_connected_supported_peers(&[ProtocolId::ConsensusRpcBcs])
            .unwrap(),
        vec![peer_network_id_2]
    );
    assert!(peers_and_metadata
        .get_connected_supported_peers(&[ProtocolId::PeerMonitoringServiceRpc])
        .unwrap()
        .is_empty());

    // Mark peer 1 as disconnecting and verify it is no longer included
    peers_and_metadata
        .update_connection_state(peer_network_id_1, ConnectionState::Disconnecting)
        .unwrap();
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        peers_and_metadata
            .get_connected_supported_peers(&[ProtocolId::MempoolDirectSend])
            .unwrap(),
        vec![peer_network_id_2]
    );
    assert!(peers_and_metadata
        .get_connected_supported_peers(&[ProtocolId::StorageServiceRpc])
        .unwrap()
        .is_empty());

    // Mark peer 2 as disconnected and verify it is no longer included
    peers_and_metadata
        .update_connection_state(peer_network_id_2, ConnectionState::Disconnected)
        .unwrap();
    assert!(peers_and_metadata
        .get_connected_peers_and_metadata()
        .unwrap()
        .is_empty());
    assert!(peers_and_metadata
        .get_connected_supported_peers(&[ProtocolId::MempoolDirectSend])
        .unwrap()
        .is_empty());

    // Reconnect both peers
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_1, connection_1.clone())
        .unwrap();
    peers_and_metadata
        .update_connection_state(peer_network_id_1, ConnectionState::Connected)
        .unwrap();
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_2, connection_2.clone())
        .unwrap();
    peers_and_metadata
        .update_connection_state(peer_network_id_2, ConnectionState::Connected)
        .unwrap();

    // Verify that removing a connection with a different connection id doesn't remove the peer
    let incorrect_connection_id = connection_1.connection_id.get_inner() + 1;
    peers_and_metadata
        .remove_peer_metadata(peer_network_id_1, incorrect_connection_id.into())
        .unwrap_err();
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        peers_and_metadata
            .get_connected_supported_peers(&[ProtocolId::MempoolDirectSend])
            .unwrap()
            .len(),
        2
    );

    // Verify that removing a connection with the same connection id works
    peers_and_metadata
        .remove_peer_metadata(peer_network_id_2, connection_2.connection_id)
        .unwrap();
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        peers_and_metadata
            .get_connected_supported_peers(&[ProtocolId::MempoolDirectSend])
            .unwrap(),
        vec![peer_network_id_1]
    );
    assert!(peers_and_metadata
        .get_connected_supported_peers(&[ProtocolId::ConsensusRpcBcs])
        .unwrap()
        .is_empty());
}

#[test]
fn test_network_client_available_peers() {
    // Create the peers and metadata container
    let network_ids = [NetworkId::Validator, NetworkId::Vfn];
    let peers_and_metadata = PeersAndMetadata::new(&network_ids);

    // Create the network client
    let direct_send_protocols = vec![
        ProtocolId::MempoolDirectSend,
        ProtocolId::ConsensusDirectSendJson,
    ];
    let rpc_protocols = vec![ProtocolId::StorageServiceRpc];
    let network_client: NetworkClient<DummyMessage> = NetworkClient::new(
        direct_send_protocols,
        rpc_protocols,
        HashMap::new(),
        peers_and_metadata.clone(),
    );

    // Verify that there are no available peers
    assert!(network_client.get_available_peers().unwrap().is_empty());

    // Create three peers and connections
    let peer_network_id_1 = PeerNetworkId::new(NetworkId::Validator, PeerId::random());
    let peer_network_id_2 = PeerNetworkId::new(NetworkId::Vfn, PeerId::random());
    let peer_network_id_3 = PeerNetworkId::new(NetworkId::Validator, PeerId::random());
    let mut connection_1 = ConnectionMetadata::mock(peer_network_id_1.peer_id());
    let mut connection_2 = ConnectionMetadata::mock(peer_network_id_2.peer_id());
    let mut connection_3 = ConnectionMetadata::mock(peer_network_id_3.peer_id());

    // Update the connections to support different protocol IDs
    let peer_protocols_1 = [ProtocolId::MempoolDirectSend, ProtocolId::StorageServiceRpc];
    let peer_protocols_2 = [
        ProtocolId::ConsensusDirectSendJson,
        ProtocolId::ConsensusRpcBcs,
    ];
    let peer_protocols_3 = [ProtocolId::ConsensusRpcBcs, ProtocolId::HealthCheckerRpc];
    connection_1.application_protocols = ProtocolIdSet::from_iter(peer_protocols_1);
    connection_2.application_protocols = ProtocolIdSet::from_iter(peer_protocols_2);
    connection_3.application_protocols = ProtocolIdSet::from_iter(peer_protocols_3);

    // Insert the connections into storage
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_1, connection_1)
        .unwrap();
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_2, connection_2.clone())
        .unwrap();
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_3, connection_3.clone())
        .unwrap();

    // Verify the correct number of available and connected peers
    let peers_and_metadata = network_client.get_peers_and_metadata();
    assert_eq!(network_client.get_available_peers().unwrap().len(), 2);
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        3
    );

    // Mark peer 3 as disconnected
    peers_and_metadata
        .update_connection_state(peer_network_id_3, ConnectionState::Disconnected)
        .unwrap();

    // Verify the correct number of available and connected peers
    assert_eq!(network_client.get_available_peers().unwrap().len(), 2);
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        2
    );

    // Remove peer 2
    peers_and_metadata
        .remove_peer_metadata(peer_network_id_2, connection_2.connection_id)
        .unwrap();

    // Verify the correct number of available and connected peers
    assert_eq!(network_client.get_available_peers().unwrap().len(), 1);
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        1
    );

    // Update peer 3 to reconnected with new protocol support
    connection_3.application_protocols = ProtocolIdSet::from_iter([ProtocolId::MempoolDirectSend]);
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_3, connection_3)
        .unwrap();
    peers_and_metadata
        .update_connection_state(peer_network_id_3, ConnectionState::Connected)
        .unwrap();

    // Verify the correct number of available and connected peers
    assert_eq!(network_client.get_available_peers().unwrap().len(), 2);
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        2
    );

    // Reconnect peer 2
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_2, connection_2)
        .unwrap();

    // Verify the correct number of available and connected peers
    assert_eq!(network_client.get_available_peers().unwrap().len(), 3);
    assert_eq!(
        peers_and_metadata
            .get_connected_peers_and_metadata()
            .unwrap()
            .len(),
        3
    );
}

#[tokio::test]
async fn test_network_client_missing_network_sender() {
    // Create the peers and metadata container
    let network_ids = [NetworkId::Validator, NetworkId::Vfn];
    let peers_and_metadata = PeersAndMetadata::new(&network_ids);

    // Create the network client
    let direct_send_protocols = vec![
        ProtocolId::MempoolDirectSend,
        ProtocolId::ConsensusDirectSendJson,
    ];
    let rpc_protocols = vec![ProtocolId::ConsensusRpcBcs];
    let network_client: NetworkClient<DummyMessage> = NetworkClient::new(
        direct_send_protocols.clone(),
        rpc_protocols.clone(),
        HashMap::new(),
        peers_and_metadata.clone(),
    );

    // Verify that there are no available peers
    assert!(network_client.get_available_peers().unwrap().is_empty());

    // Create two peers and connections
    let peer_network_id_1 = PeerNetworkId::new(NetworkId::Validator, PeerId::random());
    let peer_network_id_2 = PeerNetworkId::new(NetworkId::Vfn, PeerId::random());
    let mut connection_1 = ConnectionMetadata::mock(peer_network_id_1.peer_id());
    let mut connection_2 = ConnectionMetadata::mock(peer_network_id_2.peer_id());

    // Update the connections to support different protocol IDs
    let peer_protocols_1 = [ProtocolId::MempoolDirectSend, ProtocolId::StorageServiceRpc];
    let peer_protocols_2 = [
        ProtocolId::ConsensusDirectSendCompressed,
        ProtocolId::ConsensusRpcBcs,
    ];
    connection_1.application_protocols = ProtocolIdSet::from_iter(peer_protocols_1);
    connection_2.application_protocols = ProtocolIdSet::from_iter(peer_protocols_2);

    // Verify that sending a message to a peer without a network sender fails
    let bad_network_id = NetworkId::Public;
    let bad_peer_network_id = PeerNetworkId::new(bad_network_id, PeerId::random());
    let dummy_message = DummyMessage::new_empty();
    network_client
        .send_to_peer(dummy_message.clone(), bad_peer_network_id)
        .unwrap_err();
    network_client
        .send_to_peer_rpc(
            dummy_message.clone(),
            Duration::from_secs(MAX_MESSAGE_TIMEOUT_SECS),
            bad_peer_network_id,
        )
        .await
        .unwrap_err();

    // Verify that sending a message to all peers without a network simply logs the errors
    network_client
        .send_to_peers(dummy_message, &[bad_peer_network_id])
        .unwrap();
}

#[tokio::test]
async fn test_network_client_senders_no_matching_protocols() {
    // Create the peers and metadata container
    let network_ids = [NetworkId::Validator, NetworkId::Vfn];
    let peers_and_metadata = PeersAndMetadata::new(&network_ids);

    // Create two peers and connections
    let peer_network_id_1 = PeerNetworkId::new(NetworkId::Validator, PeerId::random());
    let peer_network_id_2 = PeerNetworkId::new(NetworkId::Vfn, PeerId::random());
    let mut connection_1 = ConnectionMetadata::mock(peer_network_id_1.peer_id());
    let mut connection_2 = ConnectionMetadata::mock(peer_network_id_2.peer_id());

    // Update the connections to support different protocol IDs
    let peer_protocols_1 = [ProtocolId::StorageServiceRpc];
    let peer_protocols_2 = [ProtocolId::ConsensusDirectSendBcs];
    connection_1.application_protocols = ProtocolIdSet::from_iter(peer_protocols_1);
    connection_2.application_protocols = ProtocolIdSet::from_iter(peer_protocols_2);

    // Create a network client with network senders
    let direct_send_protocols = vec![ProtocolId::ConsensusDirectSendBcs];
    let rpc_protocols = vec![ProtocolId::StorageServiceRpc];
    let (network_senders, _network_events, _outbound_request_receivers, _inbound_request_senders) =
        create_network_sender_and_events(&network_ids);
    let network_client: NetworkClient<DummyMessage> = NetworkClient::new(
        direct_send_protocols,
        rpc_protocols,
        network_senders,
        peers_and_metadata.clone(),
    );

    // Verify that there are no available peers
    assert!(network_client.get_available_peers().unwrap().is_empty());

    // Insert the connections into storage
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_1, connection_1)
        .unwrap();
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_2, connection_2.clone())
        .unwrap();

    // Verify that there are available peers
    assert_eq!(network_client.get_available_peers().unwrap().len(), 2);

    // Verify that sending a message to a peer without a matching protocol fails
    network_client
        .send_to_peer(DummyMessage::new_empty(), peer_network_id_1)
        .unwrap_err();
    network_client
        .send_to_peer_rpc(
            DummyMessage::new_empty(),
            Duration::from_secs(MAX_MESSAGE_TIMEOUT_SECS),
            peer_network_id_2,
        )
        .await
        .unwrap_err();
}

#[tokio::test]
async fn test_network_client_network_senders_direct_send() {
    // Create the peers and metadata container
    let network_ids = [NetworkId::Validator, NetworkId::Vfn];
    let peers_and_metadata = PeersAndMetadata::new(&network_ids);

    // Create two peers and connections
    let peer_network_id_1 = PeerNetworkId::new(NetworkId::Validator, PeerId::random());
    let peer_network_id_2 = PeerNetworkId::new(NetworkId::Vfn, PeerId::random());
    let mut connection_1 = ConnectionMetadata::mock(peer_network_id_1.peer_id());
    let mut connection_2 = ConnectionMetadata::mock(peer_network_id_2.peer_id());

    // Update the connections to support different protocol IDs
    let peer_protocols_1 = [ProtocolId::MempoolDirectSend];
    let peer_protocols_2 = [
        ProtocolId::ConsensusDirectSendCompressed,
        ProtocolId::ConsensusDirectSendJson,
        ProtocolId::ConsensusDirectSendBcs,
    ];
    connection_1.application_protocols = ProtocolIdSet::from_iter(peer_protocols_1);
    connection_2.application_protocols = ProtocolIdSet::from_iter(peer_protocols_2);

    // Create a network client with network senders
    let direct_send_protocols = vec![
        ProtocolId::MempoolDirectSend,
        ProtocolId::ConsensusDirectSendBcs,
        ProtocolId::ConsensusDirectSendJson,
        ProtocolId::ConsensusDirectSendCompressed,
    ];
    let (
        network_senders,
        network_events,
        mut outbound_request_receivers,
        mut inbound_request_senders,
    ) = create_network_sender_and_events(&network_ids);
    let network_client: NetworkClient<DummyMessage> = NetworkClient::new(
        direct_send_protocols,
        vec![],
        network_senders,
        peers_and_metadata.clone(),
    );
    let mut network_and_events = network_events.into_network_and_events();

    // Insert the connections into storage
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_1, connection_1)
        .unwrap();
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_2, connection_2.clone())
        .unwrap();

    // Verify that direct send messages are sent on matching networks and protocols
    let mut validator_network_events = network_and_events.remove(&NetworkId::Validator).unwrap();
    let mut vfn_network_events = network_and_events.remove(&NetworkId::Vfn).unwrap();
    let dummy_message = DummyMessage::new(10101);
    for peer_network_id in &[peer_network_id_1, peer_network_id_2] {
        network_client
            .send_to_peer(dummy_message.clone(), *peer_network_id)
            .unwrap();
    }
    wait_for_network_event(
        peer_network_id_1,
        &mut outbound_request_receivers,
        &mut inbound_request_senders,
        &mut validator_network_events,
        false,
        Some(ProtocolId::MempoolDirectSend),
        None,
        dummy_message.clone(),
    )
    .await;
    wait_for_network_event(
        peer_network_id_2,
        &mut outbound_request_receivers,
        &mut inbound_request_senders,
        &mut vfn_network_events,
        false,
        Some(ProtocolId::ConsensusDirectSendBcs),
        None,
        dummy_message,
    )
    .await;

    // Verify that broadcast messages are sent on matching networks and protocols
    let dummy_message = DummyMessage::new(2323);
    network_client
        .send_to_peers(dummy_message.clone(), &[
            peer_network_id_1,
            peer_network_id_2,
        ])
        .unwrap();
    wait_for_network_event(
        peer_network_id_1,
        &mut outbound_request_receivers,
        &mut inbound_request_senders,
        &mut validator_network_events,
        false,
        Some(ProtocolId::MempoolDirectSend),
        None,
        dummy_message.clone(),
    )
    .await;
    wait_for_network_event(
        peer_network_id_2,
        &mut outbound_request_receivers,
        &mut inbound_request_senders,
        &mut vfn_network_events,
        false,
        Some(ProtocolId::ConsensusDirectSendBcs),
        None,
        dummy_message,
    )
    .await;
}

#[tokio::test]
async fn test_network_client_network_senders_rpc() {
    // Create the peers and metadata container
    let network_ids = [NetworkId::Validator, NetworkId::Vfn];
    let peers_and_metadata = PeersAndMetadata::new(&network_ids);

    // Create two peers and connections
    let peer_network_id_1 = PeerNetworkId::new(NetworkId::Validator, PeerId::random());
    let peer_network_id_2 = PeerNetworkId::new(NetworkId::Vfn, PeerId::random());
    let mut connection_1 = ConnectionMetadata::mock(peer_network_id_1.peer_id());
    let mut connection_2 = ConnectionMetadata::mock(peer_network_id_2.peer_id());

    // Update the connections to support different protocol IDs
    let peer_protocols_1 = [ProtocolId::StorageServiceRpc];
    let peer_protocols_2 = [
        ProtocolId::ConsensusRpcCompressed,
        ProtocolId::ConsensusRpcJson,
        ProtocolId::ConsensusRpcBcs,
    ];
    connection_1.application_protocols = ProtocolIdSet::from_iter(peer_protocols_1);
    connection_2.application_protocols = ProtocolIdSet::from_iter(peer_protocols_2);

    // Create a network client with network senders
    let rpc_protocols = vec![
        ProtocolId::StorageServiceRpc,
        ProtocolId::ConsensusRpcJson,
        ProtocolId::ConsensusRpcBcs,
        ProtocolId::ConsensusRpcCompressed,
    ];
    let (
        network_senders,
        network_events,
        mut outbound_request_receivers,
        mut inbound_request_senders,
    ) = create_network_sender_and_events(&network_ids);
    let network_client: NetworkClient<DummyMessage> = NetworkClient::new(
        vec![],
        rpc_protocols,
        network_senders,
        peers_and_metadata.clone(),
    );
    let mut network_and_events = network_events.into_network_and_events();
    let mut validator_network_events = network_and_events.remove(&NetworkId::Validator).unwrap();
    let mut vfn_network_events = network_and_events.remove(&NetworkId::Vfn).unwrap();

    // Insert the connections into storage
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_1, connection_1)
        .unwrap();
    peers_and_metadata
        .insert_connection_metadata(peer_network_id_2, connection_2.clone())
        .unwrap();

    // Verify that rpc messages are sent on matching networks and protocols
    let dummy_message = DummyMessage::new(999);
    let rpc_timeout = Duration::from_secs(MAX_MESSAGE_TIMEOUT_SECS);
    for peer_network_id in [peer_network_id_1, peer_network_id_2] {
        let network_client = network_client.clone();
        let dummy_message = dummy_message.clone();

        // We need to spawn this on a separate thread, otherwise we'll block for the response
        tokio::spawn(async move {
            network_client
                .send_to_peer_rpc(dummy_message.clone(), rpc_timeout, peer_network_id)
                .await
                .unwrap()
        });
    }
    wait_for_network_event(
        peer_network_id_1,
        &mut outbound_request_receivers,
        &mut inbound_request_senders,
        &mut validator_network_events,
        true,
        None,
        Some(ProtocolId::StorageServiceRpc),
        dummy_message.clone(),
    )
    .await;
    wait_for_network_event(
        peer_network_id_2,
        &mut outbound_request_receivers,
        &mut inbound_request_senders,
        &mut vfn_network_events,
        true,
        None,
        Some(ProtocolId::ConsensusRpcJson),
        dummy_message,
    )
    .await;
}

// Represents a test message sent across the network
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct DummyMessage {
    pub message_contents: Option<u64>, // Dummy message contents for verification
}

impl DummyMessage {
    pub fn new(message_contents: u64) -> Self {
        Self {
            message_contents: Some(message_contents),
        }
    }

    pub fn new_empty() -> Self {
        Self {
            message_contents: None,
        }
    }
}

// Waits for a network event on the expected channels and
// verifies the message contents.
async fn wait_for_network_event(
    expected_peer_network_id: PeerNetworkId,
    outbound_request_receivers: &mut HashMap<
        NetworkId,
        aptos_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    >,
    inbound_request_senders: &mut HashMap<
        NetworkId,
        aptos_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    >,
    network_events: &mut NetworkEvents<DummyMessage>,
    is_rpc_request: bool,
    expected_direct_send_protocol_id: Option<ProtocolId>,
    expected_rpc_protocol_id: Option<ProtocolId>,
    expected_dummy_message: DummyMessage,
) {
    let expected_peer_id = expected_peer_network_id.peer_id();
    let expected_network_id = expected_peer_network_id.network_id();
    let message_wait_time = Duration::from_secs(MAX_MESSAGE_TIMEOUT_SECS);
    let channel_wait_time = Duration::from_secs(MAX_CHANNEL_TIMEOUT_SECS);

    // We first expect the message to be appear on the outbound request receivers
    let outbound_request_receiver = outbound_request_receivers
        .get_mut(&expected_network_id)
        .unwrap();
    match timeout(channel_wait_time, outbound_request_receiver.select_next_some()).await {
        Ok(peer_manager_request) => {
            let (protocol_id, peer_manager_notification) = match peer_manager_request {
                PeerManagerRequest::SendRpc(peer_id, outbound_rpc_request) => {
                    // Verify the request is correct
                    assert!(is_rpc_request);
                    assert_eq!(peer_id, expected_peer_id);
                    assert_eq!(Some(outbound_rpc_request.protocol_id), expected_rpc_protocol_id);
                    assert_eq!(outbound_rpc_request.timeout, message_wait_time);

                    // Create and return the peer manager notification
                    let inbound_rpc_request = InboundRpcRequest {
                        protocol_id: outbound_rpc_request.protocol_id,
                        data: outbound_rpc_request.data,
                        res_tx: oneshot::channel().0,
                    };
                    (outbound_rpc_request.protocol_id, PeerManagerNotification::RecvRpc(peer_id, inbound_rpc_request))
                }
                PeerManagerRequest::SendDirectSend(peer_id, message) => {
                    // Verify the request is correct
                    assert!(!is_rpc_request);
                    assert_eq!(peer_id, expected_peer_id);
                    assert_eq!(Some(message.protocol_id), expected_direct_send_protocol_id);

                    // Create and return the peer manager notification
                    (message.protocol_id, PeerManagerNotification::RecvMessage(peer_id, message))
                }
            };

            // Pass the message from the outbound request receivers to the inbound request
            // senders. This emulates network wire transfer.
            let inbound_request_sender = inbound_request_senders.get_mut(&expected_network_id).unwrap();
            inbound_request_sender.push((expected_peer_id, protocol_id), peer_manager_notification).unwrap();
        }
        Err(elapsed) => panic!(
            "Timed out while waiting to receive a message on the outbound receivers channel. Elapsed: {:?}",
            elapsed
        ),
    }

    // Now, verify the message is received by the network events and contains the correct contents
    match timeout(channel_wait_time, network_events.select_next_some()).await {
        Ok(dummy_event) => match dummy_event {
            Event::Message(peer_id, dummy_message) => {
                assert!(!is_rpc_request);
                assert_eq!(peer_id, expected_peer_id);
                assert_eq!(dummy_message, expected_dummy_message);
            },
            Event::RpcRequest(peer_id, dummy_message, protocol_id, _) => {
                assert!(is_rpc_request);
                assert_eq!(peer_id, expected_peer_id);
                assert_eq!(dummy_message, expected_dummy_message);
                assert_eq!(Some(protocol_id), expected_rpc_protocol_id);
            },
            _ => panic!("Invalid dummy event found: {:?}", dummy_event),
        },
        Err(elapsed) => panic!(
            "Timed out while waiting to receive a message on the network events receiver. Elapsed: {:?}",
            elapsed
        ),
    }
}

// Creates a set of network senders and events for the specified
// network IDs. Also returns the internal inbound and outbound
// channels for emulating network message sends across the wire.
fn create_network_sender_and_events(
    network_ids: &[NetworkId],
) -> (
    HashMap<NetworkId, NetworkSender<DummyMessage>>,
    NetworkServiceEvents<DummyMessage>,
    HashMap<NetworkId, aptos_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>>,
    HashMap<NetworkId, aptos_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
) {
    let mut network_senders = HashMap::new();
    let mut network_and_events = HashMap::new();
    let mut outbound_request_receivers = HashMap::new();
    let mut inbound_request_senders = HashMap::new();

    for network_id in network_ids {
        // Create the peer manager and connection channels
        let (inbound_request_sender, inbound_request_receiver) = create_aptos_channel();
        let (outbound_request_sender, outbound_request_receiver) = create_aptos_channel();
        let (connection_outbound_sender, _connection_outbound_receiver) = create_aptos_channel();
        let (_connection_inbound_sender, connection_inbound_receiver) = create_aptos_channel();

        // Create the network sender and events
        let network_sender = NetworkSender::new(
            PeerManagerRequestSender::new(outbound_request_sender),
            ConnectionRequestSender::new(connection_outbound_sender),
        );
        let network_events =
            NetworkEvents::new(inbound_request_receiver, connection_inbound_receiver);

        // Save the sender, events and receivers
        network_senders.insert(*network_id, network_sender);
        network_and_events.insert(*network_id, network_events);
        outbound_request_receivers.insert(*network_id, outbound_request_receiver);
        inbound_request_senders.insert(*network_id, inbound_request_sender);
    }

    // Create the network service events
    let network_service_events = NetworkServiceEvents::new(network_and_events);

    (
        network_senders,
        network_service_events,
        outbound_request_receivers,
        inbound_request_senders,
    )
}

// Returns an aptos channel for testing
fn create_aptos_channel<K: Eq + Hash + Clone, T>(
) -> (aptos_channel::Sender<K, T>, aptos_channel::Receiver<K, T>) {
    aptos_channel::new(QueueStyle::FIFO, 10, None)
}
