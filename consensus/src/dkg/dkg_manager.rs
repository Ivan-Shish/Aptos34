// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::Arc};
use aptos_types::{PeerId, transaction::SignedTransaction};
use tokio::{sync::{oneshot, mpsc}, time::Interval};
use crate::{quorum_store::batch_generator::BatchGeneratorCommand, block_storage::BlockReader};

#[derive(Debug)]
pub struct StakeDis {
    pub distribution: HashMap<PeerId, u64>,
}

#[derive(Debug)]
pub enum DKGManagerCommand {
    // parameters: new epoch, new stake distribution
    StartPVSS(u64, StakeDis),
    Shutdown(futures_channel::oneshot::Sender<()>),
}

pub struct DKGManager {
    epoch: u64,
    epoch_duration_usecs: u64,
    epoch_start_time_usecs: u64,
    pvss_transcript: Option<SignedTransaction>, // dkg todo: what is the type of pvss_transcript?
    // Block store is queried for the latest committed block's timestamp.
    block_store: Arc<dyn BlockReader + Send + Sync>,
    // channels
    batch_generator_cmd_tx: mpsc::Sender<BatchGeneratorCommand>,
}

impl DKGManager {
    pub fn new(
        epoch: u64,
        epoch_start_time_usecs: u64,
        block_store: Arc<dyn BlockReader + Send + Sync>,
        batch_generator_cmd_tx: mpsc::Sender<BatchGeneratorCommand>,
    ) -> Self {
        Self {
            epoch,
            epoch_duration_usecs: 2 * 3600 * 1_000_000, // 2 hours
            epoch_start_time_usecs,
            block_store,
            batch_generator_cmd_tx,
            pvss_transcript: None,
        }
    }

    pub async fn start(
        mut self,
        mut rx: tokio::sync::mpsc::Receiver<DKGManagerCommand>,
        mut interval: Interval,
    ) {
        loop {
            tokio::select! {
                _tick = interval.tick() => {
                    // Every 10 seconds, check if epoch has ended
                    // If epoch_start_time + epoch_interval < current_time, try sending PVSS transcript to batch generator
                    if self.epoch_start_time_usecs + self.epoch_duration_usecs < self.block_store.ordered_root().timestamp_usecs() {
                        if let Some(pvss_transcript) = self.pvss_transcript.take() {
                            self.batch_generator_cmd_tx.send(BatchGeneratorCommand::SendPVSSBatch(pvss_transcript)).await.expect("Failed to send PVSS transcript to batch generator");
                        }
                    }
                }
                Some(msg) = rx.recv() => {
                    match msg {
                        DKGManagerCommand::StartPVSS(epoch, stake_dis) => {
                            // dkg todo: start PVSS generation, once done forward to batch generator
                        }
                        DKGManagerCommand::Shutdown(ack_tx) => {
                            ack_tx.send(()).expect("Failed to send shutdown ack to round manager");
                            break;
                        }
                    }
                }
            }
        }
    }
}
