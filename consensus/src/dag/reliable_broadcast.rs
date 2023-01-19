// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::network::{NetworkSender, DagSender};
use aptos_consensus_types::common::Round;
use aptos_types::validator_signer::ValidatorSigner;
use aptos_types::validator_verifier::ValidatorVerifier;
use aptos_types::PeerId;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use claims::assert_some;
use tokio::sync::mpsc::Receiver;
use tokio::time;
use aptos_channels::aptos_channel;
use aptos_consensus_types::node::{CertifiedNode, CertifiedNodeAck, Node, SignedNodeDigest};
use crate::dag::types::{AckSet, IncrementalNodeCertificateState};
use crate::network_interface::ConsensusMsg;
use crate::round_manager::VerifiedEvent;
use futures::StreamExt;
use aptos_logger::debug;

#[allow(dead_code)]
pub(crate) enum ReliableBroadcastCommand {
    BroadcastRequest(Node),
}

#[allow(dead_code)]
pub struct ReliableBroadcast {
    my_id: PeerId,
    maybe_node: Option<Node>,
    maybe_certified_node: Option<CertifiedNode>,
    peer_round_signatures: BTreeMap<(Round, PeerId), SignedNodeDigest>,
    // vs BTreeMap<Round, BTreeMap<PeerId, ConsensusMsg>>?
    maybe_incremental_certificate_state: Option<IncrementalNodeCertificateState>,
    maybe_ack_set: Option<AckSet>,
    network_sender: NetworkSender,
    validator_verifier: ValidatorVerifier,
    validator_signer: Arc<ValidatorSigner>,
}

#[allow(dead_code)]
impl ReliableBroadcast {
    fn new(my_id: PeerId, network_sender: NetworkSender, validator_verifier: ValidatorVerifier, validator_signer: Arc<ValidatorSigner>) -> Self {
        Self {
            my_id,
            maybe_node: None,
            maybe_certified_node: None,
            //TODO: we need to persist the map and rebuild after crash
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
        self.maybe_incremental_certificate_state = Some(IncrementalNodeCertificateState::new(node.digest()));
        self.maybe_ack_set = None;
        self.network_sender.broadcast_node(node).await
    }

    // TODO: verify earlier that digest matches the node and epoch is right.
    // TODO: verify node has n-f parents(?).
    async fn handle_node_message(&mut self, node: Node) {
        match self.peer_round_signatures.get(&(node.round(), node.source())) {
            Some(signed_node_digest) => self.network_sender.send_signed_node_digest(signed_node_digest.clone(), vec![node.source()]).await,
            None => {
                let signed_node_digest = SignedNodeDigest::new(node.digest(), self.validator_signer.clone()).unwrap();
                self.peer_round_signatures.insert((node.round(), node.source()), signed_node_digest.clone());
                // TODO: persist
                self.network_sender.send_signed_node_digest(signed_node_digest, vec![node.source()]);
            }
        }
    }


    fn handle_signed_digest(&mut self, signed_node_digest: SignedNodeDigest) -> bool {
        let mut certificate_done = false;
        match self.maybe_incremental_certificate_state.as_mut() {
            None => return false,
            Some(incremental_certificate_state) => {
                incremental_certificate_state.add_signature(signed_node_digest);
                if incremental_certificate_state.ready(&self.validator_verifier) {
                    certificate_done = true;
                }
            }
        }

        if certificate_done {
            let node_certificate = self.maybe_incremental_certificate_state.take().unwrap().take(&self.validator_verifier);
            // TODO: should we persist?
            self.maybe_certified_node = Some(CertifiedNode::new(self.maybe_node.take().unwrap(), node_certificate));
            let digest = self.maybe_certified_node.as_ref().unwrap().node().digest();
            self.maybe_ack_set = Some(AckSet::new(digest));
            self.maybe_ack_set.as_mut().unwrap().add(CertifiedNodeAck::new(digest, self.my_id));
            self.network_sender.send_certified_node(self.maybe_certified_node.as_ref().unwrap().clone());
        }
        certificate_done
    }

    fn resend_node(&self) {
        assert_some!(self.maybe_node);
        assert_some!(self.maybe_incremental_certificate_state);

        let missing_peers = self.maybe_incremental_certificate_state.unwrap().missing_peers_signatures(&self.validator_verifier);




    }

    fn resend_certified_node(&self) {

    }

    pub(crate) async fn start(
        mut self,
        mut network_msg_rx: aptos_channel::Receiver<PeerId, VerifiedEvent>,
        mut command_rx: Receiver<ReliableBroadcastCommand>,
    ) {
        let mut interval = time::interval(Duration::from_millis(500)); // time out should be slightly more than one network round trip.

        loop {
            // TODO: shutdown
            tokio::select! {
                biased;

                // TODO: currently it gets low priority. Check how to avoid starvation.
                _ = interval.tick() => {
                    match (self.maybe_node.is_some(), self.maybe_certified_node.is_some()) {
                        (true, true) => { unreachable!("never send both together") },
                        (true, false) =>  self.resend_node(),
                        (false, true) => self.resend_certified_node(),
                        (false, false) => {
                            debug!("dag: reliable broadcast has nothing to resend");
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
                                interval.reset();
                            }

                        },

                        VerifiedEvent::CertifiedNodeMsg(certified_node) => {
                            // TODO: move to DAG-driver

                        },

                        VerifiedEvent::CertifiedNodeAckMsg(ack) => {
                            match self.maybe_ack_set {
                                None => {},
                                Some(ref mut ack_set) => ack_set.add(*ack),
                            }

                        },

                        _ => unreachable!("DAG gets wrong messsgaes"),
                    }

                },
            }
        }
    }
}


