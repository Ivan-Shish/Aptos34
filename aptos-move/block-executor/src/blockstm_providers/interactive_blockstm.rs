// Copyright © Aptos Foundation

use crate::{
    blockstm_providers::{LastInputOutputProvider, SchedulerProvider},
    CachePadded, ExecutionStatus, TransactionOutput, TxnInput, TxnOutput, ValidationStatus,
};
use aptos_infallible::Mutex;
use aptos_mvhashmap::types::TxnIndex;
use arc_swap::ArcSwapOption;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::{collections::HashMap, fmt::Debug};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use rayon::Scope;
use crate::blockstm_providers::RemoteDependencyListener;

pub struct InteractiveBlockStmProvider {
    txn_indices: Vec<TxnIndex>,
    positions_by_tid: DashMap<TxnIndex, usize>,
}

impl InteractiveBlockStmProvider {
    pub fn new(txn_indices: Vec<TxnIndex>) -> Self {
        let positions_by_tid = txn_indices
            .iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();
        Self {
            txn_indices,
            positions_by_tid,
        }
    }
}

impl SchedulerProvider for InteractiveBlockStmProvider {
    type TxnDependencyCollection = HashMap<TxnIndex, CachePadded<Mutex<Vec<TxnIndex>>>>;
    type TxnStatusCollection =
        HashMap<TxnIndex, CachePadded<(RwLock<ExecutionStatus>, RwLock<ValidationStatus>)>>;

    fn new_txn_dep_info(&self) -> Self::TxnDependencyCollection {
        self.txn_indices
            .iter()
            .map(|&tid| {
                let initial_dep = CachePadded::new(Mutex::new(Vec::new()));
                (tid, initial_dep)
            })
            .collect()
    }

    fn new_txn_status_provider(&self) -> Self::TxnStatusCollection {
        self.txn_indices
            .iter()
            .map(|&txn_idx| {
                let initial_status = CachePadded::new((
                    RwLock::new(ExecutionStatus::ReadyToExecute(0, None)),
                    RwLock::new(ValidationStatus::new()),
                ));
                (txn_idx, initial_status)
            })
            .collect()
    }

    fn get_txn_deps_by_tid(
        deps: &Self::TxnDependencyCollection,
        tid: TxnIndex,
    ) -> &CachePadded<Mutex<Vec<TxnIndex>>> {
        deps.get(&tid).unwrap()
    }

    fn get_txn_status_by_tid(
        status: &Self::TxnStatusCollection,
        tid: TxnIndex,
    ) -> &CachePadded<(RwLock<ExecutionStatus>, RwLock<ValidationStatus>)> {
        status.get(&tid).unwrap()
    }

    fn txn_index_right_after(&self, x: TxnIndex) -> TxnIndex {
        let pos = self.get_local_position_by_tid(x);
        if pos >= self.txn_indices.len() - 1 {
            TXN_IDX_NONE
        } else {
            self.txn_indices[pos + 1]
        }
    }

    fn all_txn_indices(&self) -> Box<dyn Iterator<Item = TxnIndex> + '_> {
        Box::new(self.txn_indices.iter().cloned())
    }

    fn get_local_position_by_tid(&self, tid: TxnIndex) -> usize {
        if tid == TXN_IDX_NONE {
            self.txn_indices.len()
        } else {
            *self.positions_by_tid.get(&tid).unwrap()
        }
    }

    fn txn_end_index(&self) -> TxnIndex {
        TXN_IDX_NONE
    }

    fn get_first_tid(&self) -> TxnIndex {
        *self.txn_indices.first().unwrap_or(&TXN_IDX_NONE)
    }

    fn num_txns(&self) -> usize {
        self.txn_indices.len()
    }
}

impl<K: Send + Sync, TO: TransactionOutput, TE: Debug + Send + Sync>
    LastInputOutputProvider<K, TO, TE> for InteractiveBlockStmProvider
{
    type CommitLockCollection = HashMap<TxnIndex, Mutex<()>>;
    type TxnLastInputCollection = HashMap<TxnIndex, CachePadded<ArcSwapOption<TxnInput<K>>>>;
    type TxnLastOutputCollection = HashMap<TxnIndex, CachePadded<ArcSwapOption<TxnOutput<TO, TE>>>>;

    fn new_txn_inputs(&self) -> Self::TxnLastInputCollection {
        self.txn_indices
            .iter()
            .map(|&tid| (tid, CachePadded::new(ArcSwapOption::empty())))
            .collect()
    }

    fn new_txn_outputs(&self) -> Self::TxnLastOutputCollection {
        self.txn_indices
            .iter()
            .map(|&tid| (tid, CachePadded::new(ArcSwapOption::empty())))
            .collect()
    }

    fn new_commit_locks(&self) -> Self::CommitLockCollection {
        self.txn_indices
            .iter()
            .map(|&tid| (tid, Mutex::new(())))
            .collect()
    }

    fn get_inputs_by_tid(
        inputs: &Self::TxnLastInputCollection,
        tid: TxnIndex,
    ) -> &CachePadded<ArcSwapOption<TxnInput<K>>> {
        inputs.get(&tid).unwrap()
    }

    fn get_outputs_by_tid(
        outputs: &Self::TxnLastOutputCollection,
        tid: TxnIndex,
    ) -> &CachePadded<ArcSwapOption<TxnOutput<TO, TE>>> {
        outputs.get(&tid).unwrap()
    }

    fn get_commit_lock_by_tid(locks: &Self::CommitLockCollection, tid: TxnIndex) -> &Mutex<()> {
        locks.get(&tid).unwrap()
    }
}

pub const TXN_IDX_NONE: TxnIndex = 0xFFFFFFFF;

impl RemoteDependencyListener for InteractiveBlockStmProvider {
    fn start_listening_to_remote_commit(&self, s: &Scope) {
        s.spawn(|_s|{
            let (_tx, rx) = mpsc::channel::<()>();
            loop {
                match rx.recv().unwrap() {
                    _ => {
                        break;
                    }
                }
            }
        });
    }
}
