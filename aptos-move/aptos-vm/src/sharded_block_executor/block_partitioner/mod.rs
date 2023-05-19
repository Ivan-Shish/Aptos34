// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

mod dependency_aware_partitioner;

use aptos_types::transaction::analyzed_transaction::AnalyzedTransaction;
use std::collections::HashMap;

pub trait BlockPartitioner: Send + Sync {
    /// Partitions the transactions into `num_shards` shards. Returns two maps, one for map of
    /// shard id to vector of accepted transaction in order of execution and other is the map
    /// of shard id to vector of rejected transactions.
    fn partition(
        &self,
        transactions: Vec<AnalyzedTransaction>,
        num_shards: usize,
    ) -> (
        HashMap<usize, Vec<AnalyzedTransaction>>,
        HashMap<usize, Vec<AnalyzedTransaction>>,
    );
}

/// An implementation of partitioner that splits the transactions into equal-sized chunks.
pub struct UniformPartitioner {}

impl BlockPartitioner for UniformPartitioner {
    fn partition(
        &self,
        transactions: Vec<AnalyzedTransaction>,
        num_shards: usize,
    ) -> (
        HashMap<usize, Vec<AnalyzedTransaction>>,
        HashMap<usize, Vec<AnalyzedTransaction>>,
    ) {
        let total_txns = transactions.len();
        if total_txns == 0 {
            return (HashMap::new(), HashMap::new());
        }
        let txns_per_shard = (total_txns as f64 / num_shards as f64).ceil() as usize;
        let mut accpeted_transactions: HashMap<usize, Vec<AnalyzedTransaction>> = HashMap::new();

        for (index, txn) in transactions.into_iter().enumerate() {
            let partion_index = get_shard_for_index(txns_per_shard, index);
            accpeted_transactions
                .entry(partion_index)
                .or_insert_with(Vec::new)
                .push(txn);
        }
        (accpeted_transactions, HashMap::new())
    }
}

fn get_shard_for_index(txns_per_shard: usize, index: usize) -> usize {
    index / txns_per_shard
}
