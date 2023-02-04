// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use aptos_consensus_types::node::{CertifiedNode, NodeMetaData};
use aptos_crypto::HashValue;
use aptos_types::validator_verifier::ValidatorVerifier;
use aptos_types::{block_info::Round, PeerId};
use std::collections::{hash_map::Entry, HashMap, HashSet};
use move_core_types::account_address::AccountAddress;

enum PeerStatus {
    Linked(Round),
    NotLinked(NodeMetaData),
}

impl PeerStatus {
    pub fn round(&self) -> Round {
        match self {
            PeerStatus::Linked(round) => *round,
            PeerStatus::NotLinked(metadata) => metadata.round(),
        }
    }

    pub fn not_linked(&self) -> bool {
        match self {
            PeerStatus::Linked(_) => false,
            PeerStatus::NotLinked(_) => true,
        }
    }

    fn metadata(self) -> NodeMetaData {
        match self {
            PeerStatus::Linked(_) => panic!("no metadata"),
            PeerStatus::NotLinked(metadata) => metadata,
        }
    }

    pub fn mark_linked(&mut self) -> Option<NodeMetaData> {
        let round = match self {
            PeerStatus::Linked(_) => None,
            PeerStatus::NotLinked(node_meta_data) => Some(node_meta_data.round()),
        };

        round.map(|r| std::mem::replace(self, PeerStatus::Linked(r)).metadata())
    }
}

///keeps track of weak links. None indicates that a (strong or weak) link was already added.
pub(crate) struct WeakLinksCreator {
    latest_nodes_metadata: Vec<PeerStatus>,
    address_to_validator_index: HashMap<PeerId, usize>,
}

impl WeakLinksCreator {
    pub fn new(verifier: &ValidatorVerifier) -> Self {
        Self {
            latest_nodes_metadata: verifier
                .address_to_validator_index()
                .iter()
                .map(|_| PeerStatus::Linked(0))
                .collect(),
            address_to_validator_index: verifier.address_to_validator_index().clone(),
        }
    }

    pub fn get_weak_links(&mut self, new_round: Round) -> HashSet<NodeMetaData> {
        self.latest_nodes_metadata
            .iter_mut()
            .filter(|node_status| node_status.not_linked() && node_status.round() < new_round - 1)
            .map(|node_status| node_status.mark_linked().unwrap())
            .collect()
    }

    pub fn update_peer_latest_node(&mut self, node_meta_data: NodeMetaData) {
        let peer_index = self
            .address_to_validator_index
            .get(&node_meta_data.source())
            .expect("invalid peer_id node metadata");
        let current_peer_round = self.latest_nodes_metadata[*peer_index].round();
        if current_peer_round < node_meta_data.round() {
            self.latest_nodes_metadata
                .insert(*peer_index, PeerStatus::NotLinked(node_meta_data));
        }
    }

    pub fn update_with_strong_links(&mut self, round: Round, strong_links: Vec<PeerId>) {
        for peer_id in strong_links {
            let index = self.address_to_validator_index.get(&peer_id).unwrap();
            debug_assert!(self.latest_nodes_metadata[*index].round() <= round);
            if self.latest_nodes_metadata[*index].round() == round {
                debug_assert!(self.latest_nodes_metadata[*index].not_linked());
                self.latest_nodes_metadata[*index].mark_linked();
            }
        }
    }
}

struct AbsentInfo {
    peer_id: PeerId,
    round: Round,
    peers_to_request: HashSet<PeerId>,
    immediate_dependencies: HashSet<HashValue>,
}

impl AbsentInfo {
    pub fn new(
        peer_id: PeerId,
        round: Round,
    ) -> Self {
        Self {
            peer_id,
            round,
            peers_to_request: HashSet::new(),
            immediate_dependencies: HashSet::new(),
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn peers_to_request(&self) -> &HashSet<PeerId> {
        &self.peers_to_request
    }

    pub fn take_immediate_dependencies(self) -> HashSet<HashValue> {
        self.immediate_dependencies
    }

    pub fn add_dependency(&mut self, digest: HashValue) {
        self.immediate_dependencies.insert(digest);
    }

    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.peers_to_request.insert(peer_id);
    }
}

struct PendingInfo {
    certified_node: CertifiedNode,
    missing_parents: HashSet<HashValue>,
    immediate_dependencies: HashSet<HashValue>,
}

impl PendingInfo {
    pub fn new(
        certified_node: CertifiedNode,
        missing_parents: HashSet<HashValue>,
        immediate_dependencies: HashSet<HashValue>,
    ) -> Self {
        Self {
            certified_node,
            missing_parents,
            immediate_dependencies,
        }
    }

    pub fn immediate_dependencies(&self) -> &HashSet<HashValue> {
        &self.immediate_dependencies
    }

    pub fn missing_parents(&self) -> &HashSet<HashValue> {
        &self.missing_parents
    }

    pub fn take(self) -> (CertifiedNode, HashSet<HashValue>) {
        (self.certified_node, self.immediate_dependencies)
    }

    pub fn take_immediate_dependencies(self) -> HashSet<HashValue> {
        self.immediate_dependencies
    }

    pub fn remove_missing_parent(&mut self, digest: HashValue) {
        self.missing_parents.remove(&digest);
    }

    pub fn ready_to_be_added(&self) -> bool {
        self.missing_parents.is_empty()
    }

    pub fn add_dependency(&mut self, digest: HashValue) {
        self.immediate_dependencies.insert(digest);
    }
}

enum MissingDagNodeStatus {
    Absent(AbsentInfo),
    Pending(PendingInfo),
}

impl MissingDagNodeStatus {
    pub fn take_node_and_dependencies(self) -> (CertifiedNode, HashSet<HashValue>) {
        match self {
            MissingDagNodeStatus::Absent(_) => { unreachable!("dag: should not call take_node_and_dependencies whan node is absent") }
            MissingDagNodeStatus::Pending(info) => info.take(),
        }
    }

    pub fn take_dependencies(self) -> HashSet<HashValue> {
        match self {
            MissingDagNodeStatus::Absent(info) => info.take_immediate_dependencies(),
            MissingDagNodeStatus::Pending(info) => info.take_immediate_dependencies(),
        }
    }

    pub fn remove_missing_parent(&mut self, digets: HashValue) {
        match self {
            MissingDagNodeStatus::Absent(info) => unreachable!("dag: node is absent, no missing parents"),
            MissingDagNodeStatus::Pending(info) => info.remove_missing_parent(digets),
        }
    }

    pub fn ready_to_be_added(&self) -> bool {
        match self {
            MissingDagNodeStatus::Absent(_) => false,
            MissingDagNodeStatus::Pending(info) => info.ready_to_be_added(),
        }
    }

    pub fn add_dependency(&mut self, digest: HashValue) {
        match self {
            MissingDagNodeStatus::Absent(info) => info.add_dependency(digest),
            MissingDagNodeStatus::Pending(info) => info.add_dependency(digest),
        }
    }

    pub fn add_peer_to_request(&mut self, peer_id: PeerId) {
        match self {
            MissingDagNodeStatus::Absent(info) => info.add_peer(peer_id),
            MissingDagNodeStatus::Pending(_) => {},
        }
    }
}

// TODO: initiate with genesys nodes
// TODO: persist all every update
pub(crate) struct Dag {
    current_round: u64,
    front: WeakLinksCreator,
    dag: Vec<HashMap<PeerId, CertifiedNode>>,
    // TODO: add genesys nodes.
    missing_nodes: HashMap<HashValue, MissingDagNodeStatus>,
}

impl Dag {
    // TODO make this pub and check also pending
    fn contains(&self, round: Round, peer_id: PeerId) -> bool {
        self.dag
            .get(round as usize)
            .map(|m| m.contains_key(&peer_id))
            == Some(true)
    }

    fn round_digests(&self, round: Round) -> Option<HashSet<HashValue>> {
        self.dag.get(round as usize).map(|m| {
            m.iter()
                .map(|(_, certified_node)| certified_node.node().digest())
                .collect()
        })
    }

    fn add_to_dag(&mut self, certified_node: CertifiedNode) {
        let round = certified_node.node().round() as usize;
        // assert!(self.dag.len() >= round - 1);

        if self.dag.len() < round {
            self.dag.push(HashMap::new());
        }
        self.dag[round].insert(certified_node.node().source(), certified_node);

        // TODO persist!

        // TODO: check if round is completed-> start new round and pass current to Bullshark. Or maybe check it makes more sense to check it at the end of the recurtion
    }

    fn update_pending_nodes(
        &mut self,
        recently_added_node_dependencies: HashSet<HashValue>,
        recently_added_node_digest: HashValue,
    ) {
        for digest in recently_added_node_dependencies {
            match self.missing_nodes.entry(digest) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().remove_missing_parent(recently_added_node_digest);
                    if entry.get_mut().ready_to_be_added() {
                        let (certified_node, dependencies) = entry.remove().take_node_and_dependencies();
                        let digest = certified_node.digest();
                        self.add_to_dag(certified_node);
                        self.update_pending_nodes(dependencies, digest);
                        // TODO: should we persist?
                    }
                }
                Entry::Vacant(_) => unreachable!("pending node is missing"),
            }
        }
    }

    fn add_peers_recursively(&mut self, digest: HashValue, source: PeerId) {

        let missing_parents = match self.missing_nodes.get(&digest).unwrap(){
            MissingDagNodeStatus::Absent(_) => HashSet::new(),
            MissingDagNodeStatus::Pending(info) => info.missing_parents().clone(),
        };

        for parent_digest in missing_parents {
            match self.missing_nodes.entry(parent_digest) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().add_peer_to_request(source);
                    self.add_peers_recursively(parent_digest, source);
                },
                Entry::Vacant(_) => unreachable!("node should exist in missing nodes"),
            };
        }
    }


    fn add_to_pending(
        &mut self,
        certified_node: CertifiedNode, // assumption that node not pending.
        missing_parents: HashSet<(PeerId, Round, HashValue)>,
    ) {
        let pending_peer_id = certified_node.node().source();
        let pending_digest = certified_node.node().digest();
        let missing_parents_digest = missing_parents.iter().map(|(_, digest)| *digest).collect();

        let dependencies = self.missing_nodes
            .remove(&pending_digest)
            .map(|status| status.take_dependencies())
            .unwrap_or(HashSet::new());
        let pending_info = PendingInfo::new(certified_node, missing_parents_digest, dependencies);
        self.missing_nodes.insert(pending_digest, MissingDagNodeStatus::Pending(pending_info));

        // TODO: Persist


        for (source, round, digest) in missing_parents {
            let status =
                self.missing_nodes
                    .entry(digest)
                    .or_insert(MissingDagNodeStatus::Absent(AbsentInfo::new(source, round)));

            status.add_dependency(pending_digest);
            status.add_peer_to_request(pending_peer_id);

            self.add_peers_recursively(digest, pending_peer_id); // Recursively update source_peers.
        }
    }
}
