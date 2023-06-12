// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dag::reliable_broadcast::{BroadcastStatus, DAGMessage, DAGNetworkSender, ReliableBroadcast},
    network_interface::ConsensusMsg::{DKGMessage, self},
};
use anyhow::bail;
use aptos_consensus_types::{common::Author, dkg_msg::DKGMsg};
use aptos_infallible::Mutex;
use aptos_logger::debug;
use aptos_types::validator_verifier::random_validator_verifier;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::sync::oneshot;

impl DAGMessage for DKGMsg {
    fn from_network_message(msg: ConsensusMsg) -> anyhow::Result<Self> {
        match msg {
            ConsensusMsg::DKGMessage(dkg_msg) => Ok(*dkg_msg),
            _ => bail!("Not DKG message, receiving {:?}", msg),
        }
    }
    fn into_network_message(self) -> ConsensusMsg {
        ConsensusMsg::DKGMessage(Box::new(self))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DKGAck;

impl DAGMessage for DKGAck {
    fn from_network_message(msg: ConsensusMsg) -> anyhow::Result<Self> {
        match msg {
            ConsensusMsg::DKGMessage(_) => Ok(DKGAck),
            _ => bail!("Not DKG ack message, receiving {:?}", msg),
        }
    }

    fn into_network_message(self) -> ConsensusMsg {
        ConsensusMsg::DKGMessage(Box::new(DKGMsg(vec![])))
    }
}

pub struct DKGBroadcastStatus {
    threshold: usize,
    received: HashSet<Author>,
}

impl BroadcastStatus for DKGBroadcastStatus {
    type Message = DKGMsg;
    type Ack = DKGAck;
    type Aggregated = HashSet<Author>;

    fn empty(receivers: Vec<Author>) -> Self {
        Self {
            threshold: receivers.len(),
            received: HashSet::new(),
        }
    }

    fn add(&mut self, peer: Author, _ack: Self::Ack) -> anyhow::Result<Option<Self::Aggregated>> {
        self.received.insert(peer);
        if self.received.len() == self.threshold {
            Ok(Some(self.received.clone()))
        } else {
            Ok(None)
        }
    }
}

// #[tokio::test]
// async fn test_dkg_reliable_broadcast() {
//     let (_, validator_verifier) = random_validator_verifier(5, None, false);
//     let validators = validator_verifier.get_ordered_account_addresses();
//     let failures = HashMap::from([(validators[0], 1), (validators[2], 3)]);
//     let sender = Arc::new(DKGSender::new(failures));
//     let rb = ReliableBroadcast::new(validators.clone(), sender);
//     let message = DKGMessage(vec![1, 2, 3]);
//     let (tx, rx) = oneshot::channel();
//     let (_cancel_tx, cancel_rx) = oneshot::channel();
//     tokio::spawn(rb.broadcast::<DKGBroadcastStatus>(message, tx, cancel_rx));
//     assert_eq!(rx.await.unwrap(), validators.into_iter().collect());
// }

// #[tokio::test]
// async fn test_dkg_reliable_broadcast_cancel() {
//     let (_, validator_verifier) = random_validator_verifier(5, None, false);
//     let validators = validator_verifier.get_ordered_account_addresses();
//     let failures = HashMap::from([(validators[0], 1), (validators[2], 3)]);
//     let sender = Arc::new(DKGSender::new(failures));
//     let rb = ReliableBroadcast::new(validators.clone(), sender);
//     let message = DKGMessage(vec![1, 2, 3]);

//     // explicit send cancel
//     let (tx, rx) = oneshot::channel();
//     let (cancel_tx, cancel_rx) = oneshot::channel();
//     cancel_tx.send(()).unwrap();
//     tokio::spawn(rb.broadcast::<DKGBroadcastStatus>(message.clone(), tx, cancel_rx));
//     assert!(rx.await.is_err());

//     // implicit drop cancel
//     let (tx, rx) = oneshot::channel();
//     let (cancel_tx, cancel_rx) = oneshot::channel();
//     drop(cancel_tx);
//     tokio::spawn(rb.broadcast::<DKGBroadcastStatus>(message, tx, cancel_rx));
//     assert!(rx.await.is_err());
// }
