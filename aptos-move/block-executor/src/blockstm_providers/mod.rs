// Copyright © Aptos Foundation

use aptos_infallible::Mutex;
use aptos_mvhashmap::types::TxnIndex;
use parking_lot::RwLock;
use std::fmt::Debug;
use arc_swap::ArcSwapOption;
use crate::{CachePadded, ExecutionStatus, TransactionOutput, TxnInput, TxnOutput, ValidationStatus};

pub mod default;
pub mod interactive_blockstm;

pub trait SchedulerProvider: Send + Sync {
    type TxnDependencyInfo : Send + Sync;
    type TxnStatusProvider : Send + Sync;
    fn new_txn_dep_info(&self) -> Self::TxnDependencyInfo;
    fn new_txn_status_provider(&self) -> Self::TxnStatusProvider;
    fn get_txn_deps_by_tid(deps: &Self::TxnDependencyInfo, tid: TxnIndex) -> &CachePadded<Mutex<Vec<TxnIndex>>>;
    fn get_txn_status_by_tid(status: &Self::TxnStatusProvider, tid: TxnIndex) -> &CachePadded<(RwLock<ExecutionStatus>, RwLock<ValidationStatus>)>;
    fn txn_index_right_after(&self, x: TxnIndex) -> TxnIndex;
    fn all_txn_indices(&self) -> Vec<TxnIndex>;
    fn get_local_position_by_tid(&self, tid: TxnIndex) -> usize;
}

pub trait LastInputOuputProvider<K, TO: TransactionOutput, TE: Debug>: Send + Sync {
    type TxnLastInputs : Send + Sync;
    type TxnLastOutputs: Send + Sync;
    type CommitLocks: Send + Sync;
    fn new_txn_inputs(&self) -> Self::TxnLastInputs;
    fn new_txn_outputs(&self) -> Self::TxnLastOutputs;
    fn new_commit_locks(&self) -> Self::CommitLocks;
    fn get_inputs_by_tid(inputs: &Self::TxnLastInputs, tid: TxnIndex) -> &CachePadded<ArcSwapOption<TxnInput<K>>>;
    fn get_outputs_by_tid(outputs: &Self::TxnLastOutputs, tid: TxnIndex) -> &CachePadded<ArcSwapOption<TxnOutput<TO, TE>>>;
    fn get_commit_lock_by_tid(locks: &Self::CommitLocks, tid: TxnIndex) -> &Mutex<()>;
}
