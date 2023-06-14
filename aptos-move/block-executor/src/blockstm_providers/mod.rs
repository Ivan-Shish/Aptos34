// Copyright © Aptos Foundation

use crate::{
    CachePadded, ExecutionStatus, TransactionOutput, TxnInput, TxnOutput, ValidationStatus,
};
use aptos_infallible::Mutex;
use aptos_mvhashmap::types::TxnIndex;
use arc_swap::ArcSwapOption;
use parking_lot::RwLock;
use std::fmt::Debug;
use rayon::Scope;

pub mod default;
pub mod interactive_blockstm;

/// This trait captures the `Scheduler`-related processes and data structures
/// where the default BlockSTM and the interactive BlockSTM differ.
pub trait SchedulerProvider: Send + Sync {
    type TxnDependencyCollection: Send + Sync;

    type TxnStatusCollection: Send + Sync;

    fn new_txn_dep_info(&self) -> Self::TxnDependencyCollection;

    fn new_txn_status_provider(&self) -> Self::TxnStatusCollection;

    fn get_txn_deps_by_tid(
        deps: &Self::TxnDependencyCollection,
        tid: TxnIndex,
    ) -> &CachePadded<Mutex<Vec<TxnIndex>>>;

    fn get_txn_status_by_tid(
        status: &Self::TxnStatusCollection,
        tid: TxnIndex,
    ) -> &CachePadded<(RwLock<ExecutionStatus>, RwLock<ValidationStatus>)>;

    /// Get the next transaction index in the list, if exists.
    fn txn_index_right_after(&self, x: TxnIndex) -> TxnIndex;

    fn all_txn_indices(&self) -> Box<dyn Iterator<Item = TxnIndex> + '_>;

    /// Get the position of the given transaction in the transaction list.
    fn get_local_position_by_tid(&self, tid: TxnIndex) -> usize;

    /// Get an invalid transaction index that represents the end of the transaction list.
    fn txn_end_index(&self) -> TxnIndex;

    /// Get the index of the first transaction.
    fn get_first_tid(&self) -> TxnIndex;
    fn num_txns(&self) -> usize;
}

/// This trait captures the `LastInputOutput`-related processes and data structures
/// where the default BlockSTM and the interactive BlockSTM differ.
pub trait LastInputOutputProvider<K, TO: TransactionOutput, TE: Debug>: Send + Sync {
    type TxnLastInputCollection: Send + Sync;
    type TxnLastOutputCollection: Send + Sync;
    type CommitLockCollection: Send + Sync;
    fn new_txn_inputs(&self) -> Self::TxnLastInputCollection;
    fn new_txn_outputs(&self) -> Self::TxnLastOutputCollection;
    fn new_commit_locks(&self) -> Self::CommitLockCollection;
    fn get_inputs_by_tid(
        inputs: &Self::TxnLastInputCollection,
        tid: TxnIndex,
    ) -> &CachePadded<ArcSwapOption<TxnInput<K>>>;
    fn get_outputs_by_tid(
        outputs: &Self::TxnLastOutputCollection,
        tid: TxnIndex,
    ) -> &CachePadded<ArcSwapOption<TxnOutput<TO, TE>>>;
    fn get_commit_lock_by_tid(locks: &Self::CommitLockCollection, tid: TxnIndex) -> &Mutex<()>;
}

pub trait RemoteDependencyListener: Send + Sync {
    fn start_listening_to_remote_commit(&self, s: &Scope);
}
