// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_types::transaction::analyzed_transaction::AnalyzedTransaction;
use std::collections::HashSet;

pub type ShardId = usize;
pub type TxnIndex = usize;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct TxnIdxWithShardId {
    pub txn_index: TxnIndex,
    pub shard_id: ShardId,
}

impl TxnIdxWithShardId {
    pub fn new(txn_index: TxnIndex, shard_id: ShardId) -> Self {
        Self {
            shard_id,
            txn_index,
        }
    }
}

#[derive(Default, Debug, Clone)]
/// Represents the dependencies of a transaction on other transactions across shards. Two types
/// of dependencies are supported:
/// 1. `depends_on`: The transaction depends on the execution of the transactions in the set. In this
/// case, the transaction can only be executed after the transactions in the set have been executed.
/// 2. `dependents`: The transactions in the set depend on the execution of the transaction. In this
/// case, the transactions in the set can only be executed after the transaction has been executed.
pub struct CrossShardDependencies {
    depends_on: HashSet<TxnIdxWithShardId>,
    dependents: HashSet<TxnIdxWithShardId>,
}

impl CrossShardDependencies {
    pub fn len(&self) -> usize {
        self.depends_on.len()
    }

    pub fn is_empty(&self) -> bool {
        self.depends_on.is_empty()
    }

    pub fn depends_on(&self) -> &HashSet<TxnIdxWithShardId> {
        &self.depends_on
    }

    pub fn is_depends_on(&self, txn_idx_with_shard_id: TxnIdxWithShardId) -> bool {
        self.depends_on.contains(&txn_idx_with_shard_id)
    }

    pub fn add_depends_on_txn(&mut self, txn_idx_with_shard_id: TxnIdxWithShardId) {
        self.depends_on
            .insert(txn_idx_with_shard_id);
    }

    pub fn add_dependent_txn(&mut self, txn_idx_with_shard_id: TxnIdxWithShardId) {
        self.dependents
            .insert(txn_idx_with_shard_id);
    }
}

#[derive(Debug, Clone)]
/// A contiguous chunk of transactions (along with their dependencies) in a block.
///
/// Each `SubBlock` represents a sequential section of transactions within a block.
/// The chunk includes the index of the first transaction relative to the block and a vector
/// of `TransactionWithDependencies` representing the transactions included in the chunk.
///
/// Illustration:
/// ```plaintext
///  Block (Split into 3 transactions chunks):
///  +----------------+------------------+------------------+
///  | Chunk 1        | Chunk 2          | Chunk 3          |
///  +----------------+------------------+------------------+
///  | Transaction 1  | Transaction 4    | Transaction 7    |
///  | Transaction 2  | Transaction 5    | Transaction 8    |
///  | Transaction 3  | Transaction 6    | Transaction 9    |
///  +----------------+------------------+------------------+
/// ```
pub struct SubBlock {
    // This is the index of first transaction relative to the block.
    pub start_index: TxnIndex,
    pub transactions: Vec<TransactionWithDependencies>,
}

impl SubBlock {
    pub fn new(start_index: TxnIndex, transactions: Vec<TransactionWithDependencies>) -> Self {
        Self {
            start_index,
            transactions,
        }
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn end_index(&self) -> TxnIndex {
        self.start_index + self.len()
    }

    pub fn transactions_with_deps(&self) -> &Vec<TransactionWithDependencies> {
        &self.transactions
    }

    pub fn add_dependent_txn(&mut self, source_index: TxnIndex, txn_idx_with_shard_id: TxnIdxWithShardId) {
        let source_txn = self.transactions.get_mut(source_index - self.start_index).unwrap();
        source_txn.add_dependents_txn(txn_idx_with_shard_id);
    }

}

#[derive(Debug, Clone)]
pub struct TransactionWithDependencies {
    pub txn: AnalyzedTransaction,
    pub cross_shard_dependencies: CrossShardDependencies,
}

impl TransactionWithDependencies {
    pub fn new(txn: AnalyzedTransaction, cross_shard_dependencies: CrossShardDependencies) -> Self {
        Self {
            txn,
            cross_shard_dependencies,
        }
    }

    #[cfg(test)]
    pub fn txn(&self) -> &AnalyzedTransaction {
        &self.txn
    }

    #[cfg(test)]
    pub fn cross_shard_dependencies(&self) -> &CrossShardDependencies {
        &self.cross_shard_dependencies
    }

    pub fn add_dependent_txn(&mut self, txn_idx_with_shard_id: TxnIdxWithShardId) {
        self.cross_shard_dependencies.add_dependent_txn(txn_idx_with_shard_id);
    }

}
