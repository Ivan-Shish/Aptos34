// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network::{DagSender, NetworkSender},
    round_manager::VerifiedEvent,
};
use aptos_channels::aptos_channel;
use aptos_consensus_types::common::Round;
use aptos_consensus_types::node::{CertifiedNode, CertifiedNodeAck, CertifiedNodeRequest};
use aptos_types::PeerId;
use futures::StreamExt;
use std::time::Duration;
use tokio::{sync::mpsc::Receiver, time};
use crate::dag::dag::Dag;

#[allow(dead_code)]
pub(crate) enum DagDriverCommand {}

// TODO: Create new a node once round is ready and pass to RB and push the round to Bullshark. Pull/get proofs from QS.
// TODO: weak links and GC.
// TODO: Timeouts and anchor election! Arc<something> and call it when needed.

#[allow(dead_code)]
pub struct DagDriver {
    my_id: PeerId,
    round: Round,
    network_sender: NetworkSender,
    // TODO: Should we clean more often than once an epoch?
    dag: Dag,

}

#[allow(dead_code)]
impl DagDriver {
    async fn remote_fetch_missing_nodes(&self) {
        for (node_meta_data, nodes_to_request) in self.dag.missing_nodes_metadata() {
            let request = CertifiedNodeRequest::new(
                node_meta_data,
                self.my_id,
            );
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

    // TODO: call self.dag.try_adding_node(certified_node) -> round ready.
    async fn handle_certified_node(&mut self, certified_node: CertifiedNode, ack_required: bool) {


        // TODO: implement the timeout logic and creating new node logic

        let digest = certified_node.digest();
        let source = certified_node.source();
        self.dag.try_add_node_and_advance_round(certified_node, false).await; // returns parents -> start new round

        if ack_required {
            let ack = CertifiedNodeAck::new(digest, self.my_id);
            self.network_sender
                .send_certified_node_ack(ack, vec![source])
                .await
        }

        todo!(); // see if round can be advanced.
    }

    #[allow(dead_code)]
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

                _ = interval.tick() => {
                self.remote_fetch_missing_nodes().await
            },

            Some(_command) = command_rx.recv() => {
                // TODO: proofs from consensus & other apps.
                // TODO: probably better to pull when time to crete new round.
            },

            Some(msg) = network_msg_rx.next() => {
                    match msg {

                        VerifiedEvent::CertifiedNodeMsg(certified_node, ack_required) => {

                            self.handle_certified_node(*certified_node, ack_required).await;

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
