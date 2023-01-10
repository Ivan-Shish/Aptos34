// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{dag::types::ReliableBroadcastCommand, network::NetworkSender};
use aptos_consensus_types::common::Round;
use aptos_crypto::bls12381;
use aptos_types::validator_signer::ValidatorSigner;
use aptos_types::validator_verifier::ValidatorVerifier;
use aptos_types::PeerId;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time;
use aptos_channels::aptos_channel;
use aptos_consensus_types::node::Node;
use crate::network_interface::ConsensusMsg;
use crate::round_manager::VerifiedEvent;

pub(crate) enum ReliableBroadcastCommand {
    BroadcastRequest(Node),
}

pub struct reliable_broadcast {
    peer_round_signatures: BTreeMap<(Round, PeerId), ConsensusMsg> // vs BTreeMap<Round, BTreeMap<PeerId, ConsensusMsg>>?

}

impl reliable_broadcast {
    fn new() -> Self {
        Self {
            //TODO: we need to persist this and rebuild after crash
            peer_round_signatures: BTreeMap::new(),
        }
    }

    pub(crate) async fn start(
        mut self,
        mut network_msg_rx: aptos_channel::Receiver<PeerId, VerifiedEvent>,
        mut rx: Receiver<ReliableBroadcastCommand>,
        validator_verifier: ValidatorVerifier,
        network_sender: NetworkSender,
        validator_signer: Arc<ValidatorSigner>,
    ) {
        let mut interval = time::interval(Duration::from_millis(100));

        loop {
            // TODO: shutdown?
            tokio::select! {
                biased;

                _ = interval.tick() => {

                },
            }
        }
    }
}


