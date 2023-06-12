// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::Arc, thread, time::Duration};
use aptos_consensus_types::{common::Author, dkg_msg::{Transcript, DKGMsg}};
use aptos_logger::info;
use aptos_types::{transaction::SignedTransaction, validator_verifier::ValidatorVerifier};
use serde::Serialize;
use tokio::{sync::{oneshot, mpsc}, time::Interval};
use crate::{
    quorum_store::batch_generator::BatchGeneratorCommand, block_storage::BlockReader,
    dkg::{dkg_reliable_broadcast::DKGBroadcastStatus},
    dag::reliable_broadcast::{ReliableBroadcast, DAGNetworkSender},
};

// the transcript size is 3.25MB
const TRANSCRIPT_SIZE: usize = 3_250_000;
const TRANSCRIPT_COMPUTE_TIME_MS: u64 = 4760;
const TRANSCRIPT_VERIFY_TIME_MS: u64 = 555;
const TRANSCRIPT_AGGREGATE_TIME_MS: u64 = 21;

#[derive(Debug)]
pub struct StakeDis {
    pub distribution: HashMap<Author, u64>,
}

#[derive(Debug)]
pub enum DKGManagerCommand {
    // parameters: new stake distribution
    ComputePVSS(StakeDis),
    ReceivePVSS(Author, Transcript),
    Shutdown(futures_channel::oneshot::Sender<()>),
}

pub struct DKGManager {
    epoch: u64,
    author: Author,
    old_validators: ValidatorVerifier,
    my_pvss: Option<Transcript>,
    // HashMap of valid PVSS transcripts received from other validators
    all_pvss: HashMap<Author, Transcript>,
    // Aggregated PVSS transcript from enough validators
    aggregated_pvss: Option<Transcript>,
    // dkg todo: add the key pair to sign the PVSS transcript
    // Channel to send the aggregated PVSS transcript to the batch generator
    batch_generator_cmd_tx: mpsc::Sender<BatchGeneratorCommand>,
    dkg_rbc: ReliableBroadcast,
}

impl DKGManager {
    pub fn new(
        epoch: u64,
        author: Author,
        old_validators: ValidatorVerifier,
        batch_generator_cmd_tx: mpsc::Sender<BatchGeneratorCommand>,
        network_sender: Arc<dyn DAGNetworkSender>,
    ) -> Self {
        let dkg_rbc = ReliableBroadcast::new(old_validators.get_ordered_account_addresses(), network_sender);
        Self {
            epoch,
            author,
            old_validators,
            my_pvss: None,
            all_pvss: HashMap::new(),
            aggregated_pvss: None,
            batch_generator_cmd_tx,
            dkg_rbc,
        }
    }

    fn compute_pvss(&mut self, stake_dis: StakeDis) -> anyhow::Result<()> {
        // dkg todo: compute pvss transcript
        thread::sleep(Duration::from_millis(TRANSCRIPT_COMPUTE_TIME_MS));
        self.my_pvss = Some(Transcript::new(TRANSCRIPT_SIZE));
        Ok(())
    }

    async fn broadcast_pvss(&self) {
        // dkg todo: reliably broadcast pvss transcript, need to ensure all validators receive it
        // waiting for the reliable broadcast implementation on main
        let validators = self.old_validators.get_ordered_account_addresses();
        let transcript_bytes = serde_json::to_vec(&self.my_pvss.clone().unwrap()).unwrap();
        let message = DKGMsg(transcript_bytes);
        let (tx, rx) = oneshot::channel();
        let (_cancel_tx, cancel_rx) = oneshot::channel();
        tokio::spawn(self.dkg_rbc.broadcast::<DKGBroadcastStatus>(message, tx, cancel_rx));
        assert_eq!(rx.await.unwrap(), validators.into_iter().collect());
    }

    fn aggregate_pvss(&self) -> Option<Transcript> {
        // dkg todo: aggregate all pvss transcripts
        thread::sleep(Duration::from_millis(TRANSCRIPT_AGGREGATE_TIME_MS));
        None
    }

    pub async fn start(
        mut self,
        mut rx: tokio::sync::mpsc::Receiver<DKGManagerCommand>,
    ) {
        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    match msg {
                        DKGManagerCommand::ComputePVSS(stake_dis) => {
                            if self.my_pvss.is_some() {
                                // If we already have a PVSS transcript for this epoch, ignore
                                continue;
                            }
                            // dkg todo: start PVSS generation, once done reliably multicast to all validators
                            if self.compute_pvss(stake_dis).is_ok() {
                                self.all_pvss.insert(self.author, self.my_pvss.clone().unwrap());
                                self.broadcast_pvss().await;
                            }
                        }
                        DKGManagerCommand::ReceivePVSS(peer, transcript) => {
                            // dkg todo: verify if the PVSS transcript is valid
                            if !self.all_pvss.contains_key(&peer) && transcript.verify(TRANSCRIPT_VERIFY_TIME_MS).is_ok() {
                                self.all_pvss.insert(peer, transcript);
                                if self.old_validators.check_voting_power(self.all_pvss.keys()).is_ok() {
                                    // dkg todo: aggregate PVSS transcripts from other validators
                                    if let Some(aggregated_pvss) = self.aggregate_pvss() {
                                        // dkg todo: generate a new transaction for the aggregated pvss transcript
                                        // dkg todo: send aggregated PVSS transcript to batch generator
                                        self.batch_generator_cmd_tx.send(BatchGeneratorCommand::SendPVSSBatch(None)).await.unwrap();
                                    }
                                }
                            }
                        }
                        DKGManagerCommand::Shutdown(ack_tx) => {
                            ack_tx.send(()).expect("Failed to send shutdown ack to round manager");
                            break;
                        }
                    }
                }
            }
        }
        info!("DKGManager of epoch {} stopped", self.epoch);
    }
}
