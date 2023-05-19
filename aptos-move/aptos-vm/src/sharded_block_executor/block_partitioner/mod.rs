// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::sharded_block_executor::transaction_dependency_graph::{DependencyGraph, Node};
use aptos_types::transaction::analyzed_transaction::AnalyzedTransaction;
use std::collections::{HashMap, HashSet};

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

#[derive(PartialEq, Eq)]
enum TransactionStatus {
    // Transaction is accepted after partitioning.
    Accepted,
    // Transaction is discarded due to creating cross-shard dependency.
    Discarded,
}

pub struct DependencyAwareUniformPartitioner {}

impl BlockPartitioner for DependencyAwareUniformPartitioner {
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
        // first build the dependency graph of the transactions
        let graph = DependencyGraph::create_dependency_graph(&transactions);

        // HashMap of transaction index to its status after partitioning.
        let mut txn_statuses = HashMap::new();
        // Senders of the transactions that are discarded. Used to discard subsequent transactions from the same sender
        // as well.
        let mut discarded_senders = HashSet::new();

        for (index, txn) in transactions.iter().enumerate() {
            // For each transaction that is dependent on this transaction, check if that is in the same
            // shard. If not, we discard this transaction.
            let current_shard_index = get_shard_for_index(txns_per_shard, index);
            let mut is_discarded = false;
            if let Some(sender) = txn.get_sender() {
                if discarded_senders.contains(&sender) {
                    is_discarded = true;
                }
            } else {
                let dependent_nodes = graph.get_dependent_nodes(Node::new(txn, index));
                if let Some(dependent_nodes) = dependent_nodes {
                    for node in dependent_nodes {
                        if let Some(txn_status) = txn_statuses.get(&node.get_index()) {
                            if *txn_status == TransactionStatus::Discarded {
                                continue;
                            }
                        }
                        let dependent_shard_index =
                            get_shard_for_index(txns_per_shard, node.get_index());
                        if dependent_shard_index != current_shard_index {
                            is_discarded = true;
                            break;
                        }
                    }
                }
            }
            if !is_discarded {
                txn_statuses.insert(index, TransactionStatus::Accepted);
            } else {
                if let Some(sender) = txn.get_sender() {
                    discarded_senders.insert(sender);
                }
                txn_statuses.insert(index, TransactionStatus::Discarded);
            }
        }
        // Iterate through the accepted and rejected transactions and create the final maps.
        let mut accpeted_transactions: HashMap<usize, Vec<AnalyzedTransaction>> = HashMap::new();
        let mut rejected_transactions: HashMap<usize, Vec<AnalyzedTransaction>> = HashMap::new();
        for (index, txn) in transactions.into_iter().enumerate() {
            let status = txn_statuses.get(&index).unwrap();
            match status {
                TransactionStatus::Accepted => {
                    let shard_index = get_shard_for_index(txns_per_shard, index);
                    accpeted_transactions
                        .entry(shard_index)
                        .or_insert_with(Vec::new)
                        .push(txn);
                },
                TransactionStatus::Discarded => {
                    let shard_index = get_shard_for_index(txns_per_shard, index);
                    rejected_transactions
                        .entry(shard_index)
                        .or_insert_with(Vec::new)
                        .push(txn);
                },
            }
        }
        (accpeted_transactions, rejected_transactions)
    }
}

fn get_shard_for_index(txns_per_shard: usize, index: usize) -> usize {
    index / txns_per_shard
}
