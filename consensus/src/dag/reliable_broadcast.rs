// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::dag::types::{AckSet, IncrementalNodeCertificateState};
use crate::network::{DagSender, NetworkSender};
use crate::round_manager::VerifiedEvent;
use aptos_channels::aptos_channel;
use aptos_consensus_types::common::Round;
use aptos_consensus_types::node::{CertifiedNode, Node, SignedNodeDigest};
use aptos_logger::{debug, info};
use aptos_types::validator_signer::ValidatorSigner;
use aptos_types::validator_verifier::ValidatorVerifier;
use aptos_types::PeerId;
use claims::assert_some;
use futures::StreamExt;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time;

#[allow(dead_code)]
pub(crate) enum ReliableBroadcastCommand {
    BroadcastRequest(Node),
}

// TODO: traits
// enum status {
//     nothing_to_send,
//     sending_node(Node, IncrementalNodeCertificateState),
//     sending_certificate(Node, AckSet)
// }

// TODO: same message for node and certifade node -> create two verified events.
// TODO: combain maybe_incremental_certificate_state and  maybe_ack_set and nothing to send to enum

#[allow(dead_code)]
pub struct ReliableBroadcast {
    my_id: PeerId,
    maybe_node: Option<Node>, // TODO: enum with the certificate?
    maybe_certified_node: Option<CertifiedNode>,
    peer_round_signatures: BTreeMap<(Round, PeerId), SignedNodeDigest>,
    // vs BTreeMap<Round, BTreeMap<PeerId, ConsensusMsg>> vs Hashset?
    maybe_incremental_certificate_state: Option<IncrementalNodeCertificateState>,
    maybe_ack_set: Option<AckSet>,
    network_sender: NetworkSender,
    validator_verifier: ValidatorVerifier,
    validator_signer: Arc<ValidatorSigner>,
}

#[allow(dead_code)]
impl ReliableBroadcast {
    fn new(
        my_id: PeerId,
        network_sender: NetworkSender,
        validator_verifier: ValidatorVerifier,
        validator_signer: Arc<ValidatorSigner>,
    ) -> Self {
        Self {
            my_id,
            maybe_node: None,
            maybe_certified_node: None,
            // TODO: we need to persist the map and rebuild after crash
            // TODO: Do we need to clean memory inside an epoc? We need to DB between epochs.
            peer_round_signatures: BTreeMap::new(),
            maybe_incremental_certificate_state: None,
            maybe_ack_set: None,
            network_sender,
            validator_verifier,
            validator_signer,
        }
    }

    async fn handle_broadcast_request(&mut self, node: Node) {
        // it is live to stop broadcasting the previous node at this point.
        self.maybe_node = Some(node.clone());
        self.maybe_incremental_certificate_state =
            Some(IncrementalNodeCertificateState::new(node.digest()));
        self.maybe_ack_set = None;
        self.network_sender.broadcast_node(node, None).await
    }

    // TODO: verify earlier that digest matches the node and epoch is right.
    // TODO: verify node has n-f parents(?).
    async fn handle_node_message(&mut self, node: Node) {
        match self
            .peer_round_signatures
            .get(&(node.round(), node.source()))
        {
            Some(signed_node_digest) => {
                self.network_sender
                    .send_signed_node_digest(signed_node_digest.clone(), vec![node.source()])
                    .await
            },
            None => {
                let signed_node_digest =
                    SignedNodeDigest::new(node.digest(), self.validator_signer.clone()).unwrap();
                self.peer_round_signatures
                    .insert((node.round(), node.source()), signed_node_digest.clone());
                // TODO: persist
                self.network_sender
                    .send_signed_node_digest(signed_node_digest, vec![node.source()])
                    .await;
            },
        }
    }

    fn handle_signed_digest(&mut self, signed_node_digest: SignedNodeDigest) -> bool {
        let mut certificate_done = false;
        match self.maybe_incremental_certificate_state.as_mut() {
            None => return false,
            Some(incremental_certificate_state) => {
                if let Err(e) = incremental_certificate_state.add_signature(signed_node_digest) {
                    info!("DAG: could not add signature, err = {:?}", e);
                }
                if incremental_certificate_state.ready(&self.validator_verifier) {
                    certificate_done = true;
                }
            },
        }

        if certificate_done {
            let node_certificate = self
                .maybe_incremental_certificate_state
                .take()
                .unwrap()
                .take(&self.validator_verifier);
            // TODO: should we persist?
            self.maybe_certified_node = Some(CertifiedNode::new(
                self.maybe_node.take().unwrap(),
                node_certificate,
            ));
            let digest = self.maybe_certified_node.as_ref().unwrap().node().digest();
            self.maybe_ack_set = Some(AckSet::new(digest));
            // self.maybe_ack_set.as_mut().unwrap().add(CertifiedNodeAck::new(digest, self.my_id));
        }
        certificate_done
    }

    async fn resend_node(&mut self) {
        assert_some!(self.maybe_node.as_ref());
        assert_some!(self.maybe_incremental_certificate_state.as_ref());

        let missing_peers = self
            .maybe_incremental_certificate_state
            .as_ref()
            .unwrap()
            .missing_peers_signatures(&self.validator_verifier);
        self.network_sender
            .broadcast_node(
                self.maybe_node.as_ref().unwrap().clone(),
                Some(missing_peers),
            )
            .await
    }

    async fn resend_certified_node(&mut self) {
        assert_some!(self.maybe_certified_node.as_ref());
        assert_some!(self.maybe_ack_set.as_ref());
        let missing_peers = self
            .maybe_ack_set
            .as_ref()
            .unwrap()
            .missing_peers(&self.validator_verifier);
        self.network_sender
            .send_certified_node(
                self.maybe_certified_node.as_ref().unwrap().clone(),
                Some(missing_peers),
                true,
            )
            .await
    }

    pub(crate) async fn start(
        mut self,
        mut network_msg_rx: aptos_channel::Receiver<PeerId, VerifiedEvent>,
        mut command_rx: Receiver<ReliableBroadcastCommand>,
    ) {
        // TODO: think about tick readability and races.
        let mut interval = time::interval(Duration::from_millis(500)); // TODO: time out should be slightly more than one network round trip.

        loop {
            // TODO: shutdown
            tokio::select! {
                    biased;

                    // TODO: currently it gets low priority. Check how to avoid starvation.
                    // TODO: enum will simplify it.
                    _ = interval.tick() => {
                        match (self.maybe_node.is_some(), self.maybe_certified_node.is_some()) {
                            (true, true) => { unreachable!("never send both together") },
                            (true, false) =>  self.resend_node().await,
                            (false, true) => self.resend_certified_node().await,
                            (false, false) => {
                                debug!("dag: reliable broadcast has nothing to resend");
                            // TODO: add a counter
                            },
            }

                    },

                    Some(command) = command_rx.recv() => {
                        match command {
                            ReliableBroadcastCommand::BroadcastRequest(node) => {
                                self.handle_broadcast_request(node).await;
                                interval.reset();
                            }
                        }
                    },

                    Some(msg) = network_msg_rx.next() => {
                        match msg {
                            VerifiedEvent::NodeMsg(node) => {
                                self.handle_node_message(*node).await
                            },

                            VerifiedEvent::SignedNodeDigestMsg(signed_node_digest) => {
                                if self.handle_signed_digest(*signed_node_digest) {
                                    self.network_sender.send_certified_node(self.maybe_certified_node.as_ref().unwrap().clone(), None, true).await;
                                    interval.reset();
                                }

                            },


                            VerifiedEvent::CertifiedNodeAckMsg(ack) => {
                                let mut clear_certified_node = false;
                                match self.maybe_ack_set {
                                    None => {},
                                    Some(ref mut ack_set) => {
                                        ack_set.add(*ack);
                                        if ack_set.missing_peers(&self.validator_verifier).is_empty(){
                                           clear_certified_node = true;
                                        }
                                    }
                                }
                                if clear_certified_node {
                                    self.maybe_ack_set = None;
                                    self.maybe_certified_node = None;
                                }

                            },

                            _ => unreachable!("reliable broadcast got wrong messsgae"),
                        }

                    },
                }
        }
    }
}
