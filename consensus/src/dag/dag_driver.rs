// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::round_manager::VerifiedEvent;
use aptos_channels::aptos_channel;
use aptos_consensus_types::common::Round;
use aptos_consensus_types::node::CertifiedNode;
use aptos_crypto::HashValue;
use aptos_types::validator_verifier::ValidatorVerifier;
use aptos_types::PeerId;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::collections::hash_map::Entry;
use tokio::{sync::mpsc::Receiver,
            time};
use std::time::Duration;
use futures::StreamExt;


#[allow(dead_code)]
pub(crate) enum DagDriverCommand {}

pub struct MissingDagNodeData {
    immediate_dependencies: HashSet<HashValue>,
    source_peers: HashSet<PeerId>,
    need_to_send_request: bool,
}

impl MissingDagNodeData {
    pub fn new(need_to_send_request: bool) -> Self {
        Self {
            immediate_dependencies: HashSet::new(),
            source_peers: HashSet::new(),
            need_to_send_request,
        }
    }

    pub fn need_to_send_request(&self) -> bool {
        self.need_to_send_request
    }

    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.source_peers.insert(peer_id);
    }

    pub fn add_dependency(&mut self, dependency: HashValue) {
        self.immediate_dependencies.insert(dependency);
    }

    pub fn disable_requests(&mut self) {
        self.need_to_send_request = false;
    }

    pub fn take_dependencies(self) -> HashSet<HashValue> {
        self.immediate_dependencies
    }
}

pub struct DagDriver {
    round: Round,
    // TODO: Should we clean more often than once an epoch?
    dag: Vec<HashMap<PeerId, CertifiedNode>>,
    // TODO: persist both maps
    // the set contains nodes' missing parents
    pending_certified_nodes: HashMap<HashValue, (CertifiedNode, HashSet<HashValue>)>,
    missing_certified_nodes: HashMap<HashValue, MissingDagNodeData>, // nodes that are missing in the dag, but might be in pending
}

impl DagDriver {
    fn contains(&self, round: Round, peer_id: PeerId) -> bool {
        if self.dag.len() >= round as usize {
            return self.dag[round as usize].contains_key(&peer_id);
        }

        return false;
    }

    fn round_digests(&self, round: Round) -> Option<HashSet<HashValue>> {
        if self.dag.len() >= round as usize {
            Some(self.dag[round as usize]
                .iter()
                .map(|(_, certofied_node)| certofied_node.node().digest())
                .collect())
        } else {
            None
        }
    }

    fn update_pending_nodes(&mut self, new_dag_node_data: MissingDagNodeData, new_dag_mode_digest: HashValue) {
        for digest in new_dag_node_data.take_dependencies() {
            match self.pending_certified_nodes.entry(digest) {
                Entry::Occupied(mut entry) => {
                    let set = &mut entry.get_mut().1;
                    // let certified_node = &entry.get().0;
                    set.remove(&new_dag_mode_digest);

                    if set.is_empty() {
                        let (certified_node, _) = entry.remove();
                        self.add_to_dag(certified_node);
                    }
                }
                Entry::Vacant(_) => unreachable!("pending node is missing"),
            }
        }
    }

    fn add_to_dag(&mut self, certified_node: CertifiedNode) {
        let round = certified_node.node().round() as usize;
        let digest = certified_node.node().digest();
        assert!(self.dag.len() >= round - 1);
        if self.dag.len() < round {
            self.dag.push(HashMap::new());
        }
        self.dag[round].insert(certified_node.node().source(), certified_node);

        // TODO persist!
        if let Some(missing_dag_node_data) = self.missing_certified_nodes.remove(&digest) {
            self.update_pending_nodes(missing_dag_node_data, digest);
        }

        // TODO: check if round is completed-> start new round and pass current to Bullshark.
    }

    fn add_peers_recursively(&mut self, digest: HashValue, source: PeerId) {
        let mut parents_to_visit = Vec::new();

        if let Some((_, missing_parents_set)) = self.pending_certified_nodes.get(&digest) {
            for parent_digest in missing_parents_set {
                match self.missing_certified_nodes.entry(*parent_digest) {
                    Entry::Occupied(mut entry) => {
                        if entry.get().need_to_send_request() {
                            entry.get_mut().add_peer(source);
                        } else {
                            parents_to_visit.push(*parent_digest);
                            // self.add_peers_recursively(*parent_digest, source);
                        }
                    }
                    Entry::Vacant(_) => unreachable!("node should exist in missing nodes"),
                }
            }
        }

        for digest in parents_to_visit {
            self.add_peers_recursively(digest, source);
        }
    }

    fn add_to_pending(&mut self, certified_node: CertifiedNode, missing_parents: HashSet<HashValue>) {
        let pending_peer_id = certified_node.node().source();
        let pending_digest = certified_node.node().digest();
        self.pending_certified_nodes.insert(pending_digest, (certified_node, missing_parents.clone()));
        // TODO: Persist

        for digest in missing_parents {
            let missing_dag_node_data = self.missing_certified_nodes.entry(digest).or_insert(
                MissingDagNodeData::new(!self.pending_certified_nodes.contains_key(&digest)));

            missing_dag_node_data.add_dependency(pending_digest);
            missing_dag_node_data.add_peer(pending_peer_id);

            self.add_peers_recursively(digest, pending_peer_id); // Recursively update source_peers.
        }
    }


    async fn handle_certified_node(&mut self, certified_node: CertifiedNode, ack_required: bool) {

        // TODO
        // if parants in the dag -> add to dag and check if the node was previously missing. Might need to requresevley call this function.
        // Otherwise, add to pending and update missing. Make sure to update peers to request from in missing.
        // Both cases, persists the node and retuen ack if needed.

        let prev_round_digest_set = match self.round_digests(certified_node.node().round() - 1) {
            None => HashSet::new(),
            Some(set) => set,
        };

        let missing_parents: HashSet<HashValue> = certified_node
            .node()
            .parents()
            .difference(&prev_round_digest_set)
            .cloned()
            .collect();

        if missing_parents.is_empty() {
            self.add_to_dag(certified_node); // TODO: should persist inside
        } else {
            self.add_to_pending(certified_node, missing_parents); // TODO: should persist inside
        }

        if ack_required {
            // TODO: send ack back
        }
    }

    pub(crate) async fn start(
        &mut self,
        mut network_msg_rx: aptos_channel::Receiver<PeerId, VerifiedEvent>,
        mut command_rx: Receiver<DagDriverCommand>,
    ) {
        let mut interval = time::interval(Duration::from_millis(500)); // time out should be slightly more than one network round trip.
        loop {
            // TODO: shutdown
            tokio::select! {
                    biased;

                    // TODO: currently it gets low priority. Check how to avoid starvation.
                    _ = interval.tick() => {
                    // send requests for all missing nodes.
                },

                Some(command) = command_rx.recv() => {
                    // TODO: proofs from consensus & other apps
                },

                Some(msg) = network_msg_rx.next() => {
                        match msg {

                            VerifiedEvent::CertifiedNodeMsg(certified_node, ack_required) => {

                                self.handle_certified_node(*certified_node, ack_required).await;

                            },

                        // handle NodeRequestMsg...

                        _ => unreachable!("reliable broadcast got wrong messsgae"),
                        }
                    },

                }
        }

        // TODO: once every tick ask for missing CertifiedNodes
    }
}
