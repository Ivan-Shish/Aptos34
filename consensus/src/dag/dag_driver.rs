// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::dag::dag::Dag;
use crate::state_replication::PayloadClient;
use crate::{
    network::{DagSender, NetworkSender},
    round_manager::VerifiedEvent,
};
use aptos_channels::aptos_channel;
use aptos_consensus_types::common::{PayloadFilter, Round};
use aptos_consensus_types::node::{
    CertifiedNode, CertifiedNodeAck, CertifiedNodeRequest, Node, NodeMetaData,
};
use aptos_types::PeerId;
use futures::StreamExt;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::{sync::mpsc::Receiver, time};

#[allow(dead_code)]
pub(crate) enum DagDriverCommand {}


#[allow(dead_code)]
pub struct DagDriver {
    epoch: u64,
    round: Round,
    my_id: PeerId,
    payload_client: Arc<dyn PayloadClient>,
    timeout: bool,
    network_sender: NetworkSender,
    // TODO: Should we clean more often than once an epoch?
    dag: Dag,
    max_node_txns: u64,
    max_node_bytes: u64,
}

#[allow(dead_code)]
impl DagDriver {
    async fn remote_fetch_missing_nodes(&self) {
        for (node_meta_data, nodes_to_request) in self.dag.missing_nodes_metadata() {
            let request = CertifiedNodeRequest::new(node_meta_data, self.my_id);
            self.network_sender
                .send_certified_node_request(request, nodes_to_request)
                .await;
        }
    }

    async fn handle_node_request(&mut self, node_request: CertifiedNodeRequest) {
        if let Some(certified_node) = self.dag.get_node(&node_request) {
            self.network_sender
                .send_certified_node(
                    certified_node.clone(),
                    Some(vec![node_request.requester()]),
                    false,
                )
                .await
        }
    }

    async fn create_node(&self, parents: HashSet<NodeMetaData>) -> Node {

        let excluded_payload = Vec::new(); // TODO
        let payload_filter = PayloadFilter::from(&excluded_payload);
        let payload = self
            .payload_client
            .pull_payload_for_dag(
                self.round,
                self.max_node_txns,
                self.max_node_bytes,
                payload_filter,
            )
            .await
            .expect("DAG: fail to retrieve payload");
        Node::new(self.epoch, self.round, self.my_id, payload, parents)
    }

    async fn try_advance_round(&mut self) -> Option<Node> {
        if let Some(parents) = self.dag
            .try_advance_round(self.timeout) {
            Some(self.create_node(parents).await)
        } else {
            None
        }
    }

    async fn handle_certified_node(&mut self, certified_node: CertifiedNode, ack_required: bool) {

        let digest = certified_node.digest();
        let source = certified_node.source();
        self.dag.try_add_node(certified_node).await;

        if ack_required {
            let ack = CertifiedNodeAck::new(digest, self.my_id);
            self.network_sender
                .send_certified_node_ack(ack, vec![source])
                .await
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn start(
        &mut self,
        mut network_msg_rx: aptos_channel::Receiver<PeerId, VerifiedEvent>,
        mut command_rx: Receiver<DagDriverCommand>,
        rb_command_tx: Sender<Node>,
    ) {
        let mut interval_missing_nodes = time::interval(Duration::from_millis(500)); // time out should be slightly more than one network round trip.
        let mut interval_timeout = time::interval(Duration::from_millis(1000)); // similar to leader timeout in our consensus
        loop {
            // TODO: shutdown
            tokio::select! {
                biased;

                _ = interval_missing_nodes.tick() => {
                self.remote_fetch_missing_nodes().await
            },

                _ = interval_timeout.tick() => {
                    if self.timeout == false {
                        self.timeout = true;
                        if let Some(node) = self.try_advance_round().await {
                            rb_command_tx.send(node).await.expect("dag: reliable broadcast receiver dropped");
                            self.timeout = false;
                            interval_timeout.reset();
                        }

                    }
                }




            Some(_command) = command_rx.recv() => {
                // TODO: proofs from consensus & other apps.
                // TODO: probably better to pull when time to crete new round (similarly to current code).
            },

            Some(msg) = network_msg_rx.next() => {
                    match msg {

                        VerifiedEvent::CertifiedNodeMsg(certified_node, ack_required) => {

                            self.handle_certified_node(*certified_node, ack_required).await;
                            if let Some(node) = self.try_advance_round().await {
                                rb_command_tx.send(node).await.expect("dag: reliable broadcast receiver dropped");
                                self.timeout = false;
                                interval_timeout.reset();
                            }

                        },

                        VerifiedEvent::CertifiedNodeRequestMsg(node_request) => {
                            self.handle_node_request(*node_request).await;
                    }

                    _ => unreachable!("reliable broadcast got wrong messsgae"),
                    }
                },

            }
        }
    }
}
