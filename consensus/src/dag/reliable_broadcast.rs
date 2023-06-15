// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{dag::DAGNetworkMessage, network_interface::ConsensusMsg};
use anyhow::bail;
use aptos_consensus_types::common::Author;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, sync::Arc, time::Duration};

pub trait DAGMessage: Sized + Clone + Serialize + DeserializeOwned {
    fn epoch(&self) -> u64;

    fn from_network_message(msg: ConsensusMsg) -> anyhow::Result<Self> {
        match msg {
            ConsensusMsg::DAGMessage(msg) => Ok(bcs::from_bytes(&msg.data)?),
            _ => bail!("unexpected consensus message type in dag"),
        }
    }

    fn into_network_message(self) -> ConsensusMsg {
        ConsensusMsg::DAGMessage(DAGNetworkMessage {
            epoch: self.epoch(),
            data: bcs::to_bytes(&self).unwrap(),
        })
    }
}

pub trait BroadcastStatus {
    type Ack: DAGMessage;
    type Aggregated;
    type Message: DAGMessage;

    fn add(&mut self, peer: Author, ack: Self::Ack) -> anyhow::Result<Option<Self::Aggregated>>;
}

#[async_trait]
pub trait DAGNetworkSender: Send + Sync {
    async fn send_rpc(
        &self,
        receiver: Author,
        message: ConsensusMsg,
        timeout: Duration,
    ) -> anyhow::Result<ConsensusMsg>;
}

pub struct ReliableBroadcast {
    validators: Vec<Author>,
    network_sender: Arc<dyn DAGNetworkSender>,
}

impl ReliableBroadcast {
    pub fn new(validators: Vec<Author>, network_sender: Arc<dyn DAGNetworkSender>) -> Self {
        Self {
            validators,
            network_sender,
        }
    }

    pub fn broadcast<S: BroadcastStatus>(
        &self,
        message: S::Message,
        mut aggregating: S,
    ) -> impl Future<Output = S::Aggregated> {
        let receivers: Vec<_> = self.validators.clone();
        let network_sender = self.network_sender.clone();
        async move {
            let mut fut = FuturesUnordered::new();
            let send_message = |receiver, message| {
                let network_sender = network_sender.clone();
                async move {
                    (
                        receiver,
                        network_sender
                            .send_rpc(receiver, message, Duration::from_millis(500))
                            .await,
                    )
                }
            };
            let network_message = message.into_network_message();
            for receiver in receivers {
                fut.push(send_message(receiver, network_message.clone()));
            }
            while let Some((receiver, result)) = fut.next().await {
                match result {
                    Ok(msg) => {
                        if let Ok(ack) = S::Ack::from_network_message(msg) {
                            if let Ok(Some(aggregated)) = aggregating.add(receiver, ack) {
                                return aggregated;
                            }
                        }
                    },
                    Err(_) => fut.push(send_message(receiver, network_message.clone())),
                }
            }
            unreachable!("Should aggregate with all responses");
        }
    }
}
