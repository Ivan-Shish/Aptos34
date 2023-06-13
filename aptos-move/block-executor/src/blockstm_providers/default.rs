// Copyright © Aptos Foundation

use crate::{
    blockstm_providers::{LastInputOuputProvider, SchedulerProvider},
    CachePadded, ExecutionStatus, TransactionOutput, TxnInput, TxnOutput, ValidationStatus,
};
use aptos_infallible::Mutex;
use aptos_mvhashmap::types::TxnIndex;
use arc_swap::ArcSwapOption;
use parking_lot::RwLock;
use std::fmt::Debug;

pub struct DefaultProvider {
    num_txns: TxnIndex,
}

impl DefaultProvider {
    pub fn new(num_txns: usize) -> Self {
        Self {
            num_txns: num_txns as TxnIndex,
        }
    }
}

impl SchedulerProvider for DefaultProvider {
    type TxnDependencyInfo = Vec<CachePadded<Mutex<Vec<TxnIndex>>>>;
    type TxnStatusProvider = Vec<CachePadded<(RwLock<ExecutionStatus>, RwLock<ValidationStatus>)>>;

    fn new_txn_dep_info(&self) -> Self::TxnDependencyInfo {
        (0..self.num_txns)
            .map(|_| CachePadded::new(Mutex::new(Vec::new())))
            .collect()
    }

    fn new_txn_status_provider(&self) -> Self::TxnStatusProvider {
        (0..self.num_txns)
            .map(|_| {
                CachePadded::new((
                    RwLock::new(ExecutionStatus::ReadyToExecute(0, None)),
                    RwLock::new(ValidationStatus::new()),
                ))
            })
            .collect()
    }

    fn get_txn_deps_by_tid(
        deps: &Self::TxnDependencyInfo,
        tid: TxnIndex,
    ) -> &CachePadded<Mutex<Vec<TxnIndex>>> {
        &deps[tid as usize]
    }

    fn get_txn_status_by_tid(
        status: &Self::TxnStatusProvider,
        tid: TxnIndex,
    ) -> &CachePadded<(RwLock<ExecutionStatus>, RwLock<ValidationStatus>)> {
        &status[tid as usize]
    }

    fn txn_index_right_after(&self, x: TxnIndex) -> TxnIndex {
        x + 1
    }

    fn all_txn_indices(&self) -> Vec<TxnIndex> {
        (0..self.num_txns).collect()
    }

    fn get_local_position_by_tid(&self, tid: TxnIndex) -> usize {
        (tid + 1) as usize
    }

    fn txn_end_index(&self) -> TxnIndex {
        self.num_txns
    }

    fn get_first_tid(&self) -> TxnIndex {
        0
    }

    fn num_txns(&self) -> usize {
        self.num_txns as usize
    }
}

impl<K: Send + Sync, TO: TransactionOutput, TE: Debug + Send + Sync>
    LastInputOuputProvider<K, TO, TE> for DefaultProvider
{
    type CommitLocks = Vec<Mutex<()>>;
    type TxnLastInputs = Vec<CachePadded<ArcSwapOption<TxnInput<K>>>>;
    type TxnLastOutputs = Vec<CachePadded<ArcSwapOption<TxnOutput<TO, TE>>>>;

    fn new_txn_inputs(&self) -> Self::TxnLastInputs {
        (0..self.num_txns)
            .map(|_| CachePadded::new(ArcSwapOption::empty()))
            .collect()
    }

    fn new_txn_outputs(&self) -> Self::TxnLastOutputs {
        (0..self.num_txns)
            .map(|_| CachePadded::new(ArcSwapOption::empty()))
            .collect()
    }

    fn new_commit_locks(&self) -> Self::CommitLocks {
        (0..self.num_txns).map(|_| Mutex::new(())).collect()
    }

    fn get_inputs_by_tid(
        inputs: &Self::TxnLastInputs,
        tid: TxnIndex,
    ) -> &CachePadded<ArcSwapOption<TxnInput<K>>> {
        &inputs[tid as usize]
    }

    fn get_outputs_by_tid(
        outputs: &Self::TxnLastOutputs,
        tid: TxnIndex,
    ) -> &CachePadded<ArcSwapOption<TxnOutput<TO, TE>>> {
        &outputs[tid as usize]
    }

    fn get_commit_lock_by_tid(locks: &Self::CommitLocks, tid: TxnIndex) -> &Mutex<()> {
        &locks[tid as usize]
    }
}
