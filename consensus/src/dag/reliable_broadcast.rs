
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

pub struct reliable_broadcast {
    //TODO: Consider storing the message instead of the signature.
    peer_round_signatures: BTreeMap<Round, BTreeMap<PeerId, bls12381::Signature>>,
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


