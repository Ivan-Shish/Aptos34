// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{metrics::TIMER, proof_fetcher::ProofFetcher, DbReader};
use anyhow::{anyhow, Result};
use aptos_crypto::{hash::CryptoHash, HashValue};
use aptos_logger::{error, sample, sample::SampleRate};
use aptos_types::{
    proof::SparseMerkleProofExt,
    state_store::{state_key::StateKey, state_value::StateValue},
    transaction::Version,
};
use aptos_vm::AptosVM;
use crossbeam_channel::{unbounded, Receiver, Sender};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

static IO_POOL: Lazy<rayon::ThreadPool> = Lazy::new(|| {
    rayon::ThreadPoolBuilder::new()
        .num_threads(AptosVM::get_num_proof_reading_threads())
        .thread_name(|index| format!("proof_reader_{}", index))
        .build()
        .unwrap()
});

struct Proof {
    state_key_hash: HashValue,
    proof: SparseMerkleProofExt,
}

pub struct AsyncProofFetcher {
    reader: Arc<dyn DbReader>,
    data_sender: Sender<Proof>,
    data_receiver: Receiver<Proof>,
    num_proofs_to_read: AtomicUsize,
}

impl AsyncProofFetcher {
    pub fn new(reader: Arc<dyn DbReader>) -> Self {
        let (data_sender, data_receiver) = unbounded();

        Self {
            reader,
            data_sender,
            data_receiver,
            num_proofs_to_read: AtomicUsize::new(0),
        }
    }
}

impl ProofFetcher for AsyncProofFetcher {
    fn fetch_state_value_and_proof(
        &self,
        state_key: &StateKey,
        version: Version,
        root_hash: Option<HashValue>,
    ) -> Result<(Option<StateValue>, Option<SparseMerkleProofExt>)> {
        let _timer = TIMER
            .with_label_values(&["async_proof_fetcher_fetch"])
            .start_timer();
        let value = self.reader.get_state_value_by_version(state_key, version)?;
        Ok((value, None))
    }

    fn get_proof_cache(&self) -> HashMap<HashValue, SparseMerkleProofExt> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockDbReaderWriter;
    use aptos_types::state_store::state_key::StateKeyInner;
    use assert_unordered::assert_eq_unordered;

    #[test]
    fn test_fetch() {
        let fetcher = AsyncProofFetcher::new(Arc::new(MockDbReaderWriter));
        let mut expected_key_hashes = vec![];
        for i in 0..10 {
            let state_key: StateKey = StateKey::raw(format!("test_key_{}", i).into_bytes());
            expected_key_hashes.push(state_key.hash());
            let result = fetcher
                .fetch_state_value_and_proof(&state_key, 0, None)
                .expect("Should not fail.");
            let expected_value = StateValue::from(match state_key.into_inner() {
                StateKeyInner::Raw(key) => key,
                _ => unreachable!(),
            });
            assert_eq!(result.0, Some(expected_value));
            assert!(result.1.is_none());
        }

        let proofs = fetcher.get_proof_cache();
        assert_eq!(proofs.len(), 10);
        assert_eq_unordered!(proofs.into_keys().collect::<Vec<_>>(), expected_key_hashes);
    }
}
