// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::peer_manager::{ConnectionRequestSender, PeerManagerRequestSender};
use crate::protocols::network::NetworkSender;
use crate::protocols::wire::handshake::v1::ProtocolId;
use crate::{
    application::{
        storage::{LockingHashMap, PeerMetadataStorage},
        types::{PeerInfo, PeerState},
    },
    error::NetworkError,
    protocols::network::{Message, RpcError},
};
use aptos_config::network_id::{NetworkId, PeerNetworkId};
use aptos_types::network_address::NetworkAddress;
use aptos_types::PeerId;
use async_trait::async_trait;
use itertools::Itertools;
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, time::Duration};

/// A simple interface offered by the networking stack to each application (e.g., consensus,
/// state sync, mempool, etc.). This interface provides basic support for sending messages,
/// receiving messages, disconnecting from peers, notifying the network stack of new peers
/// and managing application specific metadata for each peer (e.g., peer scores and liveness).
// TODO: API calls for managing metadata, updating state, etc.
#[async_trait]
pub trait ApplicationNetworkInterfaceTrait<Message: Send>: Clone {
    /// Adds the given peer list to the set of discovered peers
    /// that can potentially be dialed for future connections.
    async fn add_peers_to_discovery(
        &self,
        _peers: &[(PeerNetworkId, NetworkAddress)],
    ) -> Result<(), NetworkError> {
        unimplemented!()
    }

    /// Requests that the network connection for the specified peer
    /// is disconnected.
    // TODO: support disconnect reasons.
    async fn disconnect_from_peer(&self, _peer: PeerNetworkId) -> Result<(), NetworkError> {
        unimplemented!()
    }

    /// Sends the given message to the specified peer. Note: this
    /// method does not guarantee message delivery or handle responses.
    async fn send_to_peer(
        &self,
        _message: Message,
        _peer: PeerNetworkId,
    ) -> Result<(), NetworkError> {
        unimplemented!()
    }

    /// Sends the given message to each peer in the specified peer list.
    /// Note: this method does not guarantee message delivery or handle responses.
    async fn send_to_peers(
        &self,
        _message: Message,
        _peers: &[PeerNetworkId],
    ) -> Result<(), NetworkError> {
        unimplemented!()
    }

    /// Sends the given message to the specified peer with the corresponding
    /// timeout. Awaits a response from the peer, or hits the timeout
    /// (whichever occurs first).
    async fn send_to_peer_rpc(
        &self,
        _message: Message,
        _rpc_timeout: Duration,
        _peer: PeerNetworkId,
    ) -> Result<Message, RpcError> {
        unimplemented!()
    }
}

/// A network component that can be used by applications (e.g., consensus,
/// state sync and mempool, etc.) to interact with the network and other peers.
#[derive(Clone, Debug)]
pub struct ApplicationNetworkInterface<Message: Send> {
    network_sender: NetworkSender<Message>,
    protocol_id: ProtocolId,
}

impl<Message: Send> ApplicationNetworkInterface<Message> {
    fn new(
        protocol_id: ProtocolId,
        peer_manager_request_sender: PeerManagerRequestSender,
        connection_request_sender: ConnectionRequestSender,
    ) -> Self {
        Self {
            protocol_id,
            network_sender: NetworkSender::new(
                peer_manager_request_sender,
                connection_request_sender,
            ),
        }
    }
}

#[async_trait]
impl<Message: Send> ApplicationNetworkInterfaceTrait<Message>
    for ApplicationNetworkInterface<Message>
{
    async fn add_peers_to_discovery(
        &self,
        _peers: &[(PeerNetworkId, NetworkAddress)],
    ) -> Result<(), NetworkError> {
        unimplemented!("Coming soon!")
    }

    async fn disconnect_from_peer(&self, peer: PeerNetworkId) -> Result<(), NetworkError> {
        self.network_sender.disconnect_peer(peer.peer_id())
    }

    async fn send_to_peer(
        &self,
        message: Message,
        peer: PeerNetworkId,
    ) -> Result<(), NetworkError> {
        self.network_sender
            .send_to(peer.peer_id(), self.protocol_id, message)
    }

    async fn send_to_peers(
        &self,
        message: Message,
        peers: &[PeerNetworkId],
    ) -> Result<(), NetworkError> {
        let peers = peers
            .iter()
            .map(|peer_network_id| peer_network_id.peer_id());
        self.network_sender
            .send_to_many(peers, self.protocol_id, message)
    }

    async fn send_to_peer_rpc(
        &self,
        message: Message,
        rpc_timeout: Duration,
        peer: PeerNetworkId,
    ) -> Result<Message, RpcError> {
        // TODO: how do we handle the network id??
        self.network_sender
            .send_rpc(peer.peer_id(), self.protocol_id, message, rpc_timeout)
    }
}

/// A generic `NetworkInterface` for applications to connect to networking
///
/// Each application would implement their own `NetworkInterface`.  This would hold `AppData` specific
/// to the application as well as a specific `Sender` for cloning across threads and sending requests.
#[async_trait]
pub trait NetworkInterface<TMessage: Message + Send, NetworkSender> {
    /// The application specific key for `AppData`
    type AppDataKey: Clone + Debug + Eq + Hash;
    /// The application specific data to be stored
    type AppData: Clone + Debug;

    /// Provides the `PeerMetadataStorage` for other functions.  Not expected to be used externally.
    fn peer_metadata_storage(&self) -> &PeerMetadataStorage;

    /// Give a copy of the sender for the network
    fn sender(&self) -> NetworkSender;

    /// Retrieve only connected peers
    fn connected_peers(&self, network_id: NetworkId) -> HashMap<PeerNetworkId, PeerInfo> {
        self.filtered_peers(network_id, |(_, peer_info)| {
            peer_info.status == PeerState::Connected
        })
    }

    /// Filter peers with according `filter`
    fn filtered_peers<F: FnMut(&(&PeerId, &PeerInfo)) -> bool>(
        &self,
        network_id: NetworkId,
        filter: F,
    ) -> HashMap<PeerNetworkId, PeerInfo> {
        self.peer_metadata_storage()
            .read_filtered(network_id, filter)
    }

    /// Retrieve PeerInfo for the node
    fn peers(&self, network_id: NetworkId) -> HashMap<PeerNetworkId, PeerInfo> {
        self.peer_metadata_storage().read_all(network_id)
    }

    /// Application specific data interface
    fn app_data(&self) -> &LockingHashMap<Self::AppDataKey, Self::AppData>;
}

#[derive(Clone, Debug)]
pub struct MultiNetworkSender<
    TMessage: Message + Send,
    Sender: ApplicationNetworkSender<TMessage> + Send,
> {
    senders: HashMap<NetworkId, Sender>,
    _phantom: PhantomData<TMessage>,
}

impl<TMessage: Clone + Message + Send, Sender: ApplicationNetworkSender<TMessage> + Send>
    MultiNetworkSender<TMessage, Sender>
{
    pub fn new(senders: HashMap<NetworkId, Sender>) -> Self {
        MultiNetworkSender {
            senders,
            _phantom: Default::default(),
        }
    }

    fn sender(&self, network_id: &NetworkId) -> &Sender {
        self.senders.get(network_id).expect("Unknown NetworkId")
    }

    pub fn send_to(&self, recipient: PeerNetworkId, message: TMessage) -> Result<(), NetworkError> {
        self.sender(&recipient.network_id())
            .send_to(recipient.peer_id(), message)
    }

    pub fn send_to_many(
        &self,
        recipients: impl Iterator<Item = PeerNetworkId>,
        message: TMessage,
    ) -> Result<(), NetworkError> {
        for (network_id, recipients) in
            &recipients.group_by(|peer_network_id| peer_network_id.network_id())
        {
            let sender = self.sender(&network_id);
            let peer_ids = recipients.map(|peer_network_id| peer_network_id.peer_id());
            sender.send_to_many(peer_ids, message.clone())?;
        }
        Ok(())
    }

    pub async fn send_rpc(
        &self,
        recipient: PeerNetworkId,
        req_msg: TMessage,
        timeout: Duration,
    ) -> Result<TMessage, RpcError> {
        self.sender(&recipient.network_id())
            .send_rpc(recipient.peer_id(), req_msg, timeout)
            .await
    }
}
