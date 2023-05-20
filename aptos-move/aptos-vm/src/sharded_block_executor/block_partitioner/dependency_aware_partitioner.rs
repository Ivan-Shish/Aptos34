// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::sharded_block_executor::{
    block_partitioner,
    block_partitioner::BlockPartitioner,
    transaction_dependency_graph::{DependencyGraph, Node},
};
use aptos_types::transaction::analyzed_transaction::AnalyzedTransaction;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use move_core_types::account_address::AccountAddress;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum PartitioningStatus {
    // Transaction is accepted after partitioning.
    Accepted,
    // Transaction is discarded due to creating cross-shard dependency.
    Discarded,
}

pub struct DependencyAwareUniformPartitioner {}

struct PartitionerState<'a> {
    transactions: &'a [AnalyzedTransaction],
    // HashMap of transaction index to its status after partitioning.
    // Dependency graph
    graph: DependencyGraph<'a>,
    txns_per_shard: usize,
}

// Maintains the state of the depedency-aware partitioner.
impl<'a> PartitionerState<'a> {
    fn new(transactions: &'a [AnalyzedTransaction], num_shards: usize) -> Self {
        let graph = DependencyGraph::create(transactions);
        let txns_per_shard = (transactions.len() as f64 / num_shards as f64).ceil() as usize;
        Self {
            transactions,
            graph,
            txns_per_shard,
        }
    }

    pub fn discard_conflicting_cross_shard_txns(&'a self) -> DashMap<usize, PartitioningStatus> {
        let txn_statuses = DashMap::new();
        // Discarded senders to the minimum index of the transaction that was discarded.
        let discarded_senders: DashMap<AccountAddress, usize> = DashMap::new();
        // We traverse the transactions in reverse order because we want to prioritize the transactions
        // at the beginning of the block.
        for (index, txn) in self.transactions.iter().enumerate().rev() {
            let current_shard_index =
                block_partitioner::get_shard_for_index(self.txns_per_shard, index);
            let mut is_discarded = false;

            // For each transaction that is dependent on this transaction, check if that is in the same
            // shard. If not, we discard this transaction.
            let dependent_nodes = self.graph.get_dependent_nodes(Node::new(txn, index));
            if let Some(dependent_nodes) = dependent_nodes {
                for node in dependent_nodes.iter() {
                    if let Some(txn_status) = txn_statuses.get(&node.index()) {
                        if *txn_status == PartitioningStatus::Discarded {
                            continue;
                        }
                    }
                    let dependent_shard_index =
                        block_partitioner::get_shard_for_index(self.txns_per_shard, node.index());
                    if dependent_shard_index != current_shard_index {
                        is_discarded = true;
                        break;
                    }
                }
            }
            if !is_discarded {
                txn_statuses.insert(index, PartitioningStatus::Accepted);
            } else {
                discarded_senders.entry(txn.get_sender().unwrap()).and_modify(|entry| {
                    if *entry > index {
                        *entry = index;
                    }
                }).or_insert(index);
                txn_statuses.insert(index, PartitioningStatus::Discarded);
            }
        }
        txn_statuses
    }

    pub fn discard_txn_from_discarded_senders(
        &self,
        txn_statuses: &DashMap<usize, PartitioningStatus>,
    ) {
        // Discard transactions based on the sender.
        let mut discarded_senders = HashSet::new();
        for (index, txn) in self.transactions.iter().enumerate() {
            // check if the sender of the transaction is already discarded.
            // If yes, discard this transaction as well.
            if let Some(sender) = txn.get_sender() {
                if discarded_senders.contains(&sender) {
                    txn_statuses.insert(index, PartitioningStatus::Discarded);
                    continue;
                }
            }
            let entry = txn_statuses.get(&index).unwrap();
            if entry.value() == &PartitioningStatus::Discarded {
                if let Some(sender) = txn.get_sender() {
                    discarded_senders.insert(sender);
                }
            }
        }
    }
}

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
        let txns_per_shard = (transactions.len() as f64 / num_shards as f64).ceil() as usize;

        let partitioner_state = PartitionerState::new(&transactions, num_shards);
        let txn_statuses = partitioner_state.discard_conflicting_cross_shard_txns();
        partitioner_state.discard_txn_from_discarded_senders(&txn_statuses);

        let mut accepted_txns = HashMap::new();
        let mut discarded_txns = HashMap::new();
        for (index, txn) in transactions.into_iter().enumerate() {
            let entry = txn_statuses.get(&index).unwrap();
            let shard_index = block_partitioner::get_shard_for_index(txns_per_shard, index);
            if entry.value() == &PartitioningStatus::Accepted {
                accepted_txns
                    .entry(shard_index)
                    .or_insert_with(Vec::new)
                    .push(txn);
            } else {
                discarded_txns
                    .entry(shard_index)
                    .or_insert_with(Vec::new)
                    .push(txn);
            }
        }
        (accepted_txns, discarded_txns)
    }
}

#[cfg(test)]
mod tests {
    use crate::sharded_block_executor::{
        block_partitioner::{
            dependency_aware_partitioner::{DependencyAwareUniformPartitioner, PartitioningStatus},
            get_shard_for_index, BlockPartitioner,
        },
        test_utils::{
            create_non_conflicting_p2p_transaction, create_signed_p2p_transaction,
            generate_test_account,
        },
    };
    use aptos_types::transaction::analyzed_transaction::AnalyzedTransaction;
    use rand::{rngs::OsRng, Rng};
    use std::collections::HashMap;

    fn verify_txn_statuses(
        txn_statuses: &HashMap<usize, PartitioningStatus>,
        expected_txn_statuses: &HashMap<usize, PartitioningStatus>,
    ) {
        assert_eq!(txn_statuses.len(), expected_txn_statuses.len());
        for (index, status) in txn_statuses {
            assert_eq!(status, expected_txn_statuses.get(index).unwrap());
        }
    }

    fn verify_txn_shards(
        orig_txns: &Vec<AnalyzedTransaction>,
        accepted_txns: &HashMap<usize, Vec<AnalyzedTransaction>>,
        rejected_txns: &HashMap<usize, Vec<AnalyzedTransaction>>,
        num_shards: usize,
    ) {
        // create a map of transaction to its shard index.
        let mut txn_shard_map = HashMap::new();
        for (shard_index, txns) in accepted_txns {
            for txn in txns {
                txn_shard_map.insert(txn, *shard_index);
            }
        }
        for (shard_index, txns) in rejected_txns {
            for txn in txns {
                txn_shard_map.insert(txn, *shard_index);
            }
        }
        let txns_per_shard = (orig_txns.len() as f64 / num_shards as f64).ceil() as usize;
        // verify that all the transactions are present in the map.
        assert_eq!(txn_shard_map.len(), orig_txns.len());
        for (index, txn) in orig_txns.iter().enumerate() {
            assert_eq!(
                get_shard_for_index(txns_per_shard, index),
                *txn_shard_map.get(txn).unwrap()
            );
        }
    }

    fn populate_txn_statuses(
        orig_txns: &[AnalyzedTransaction],
        txns_map: &HashMap<usize, Vec<AnalyzedTransaction>>,
        txn_statuses: &mut HashMap<usize, PartitioningStatus>,
        status: PartitioningStatus,
    ) {
        for txns in txns_map.values() {
            for txn in txns {
                let index = orig_txns.iter().position(|x| x == txn).unwrap();
                txn_statuses.insert(index, status);
            }
        }
    }

    fn verify_txn_statuses_and_shards(
        orig_txns: &Vec<AnalyzedTransaction>,
        accepted_txns: &HashMap<usize, Vec<AnalyzedTransaction>>,
        rejected_txns: &HashMap<usize, Vec<AnalyzedTransaction>>,
        expected_txn_statuses: &HashMap<usize, PartitioningStatus>,
        num_shards: usize,
    ) {
        let mut txn_statuses = HashMap::new();

        populate_txn_statuses(
            orig_txns,
            accepted_txns,
            &mut txn_statuses,
            PartitioningStatus::Accepted,
        );

        populate_txn_statuses(
            orig_txns,
            rejected_txns,
            &mut txn_statuses,
            PartitioningStatus::Discarded,
        );

        verify_txn_statuses(&txn_statuses, expected_txn_statuses);
        verify_txn_shards(orig_txns, accepted_txns, rejected_txns, num_shards);
    }

    #[test]
    // Test that the partitioner works correctly for a single sender and multiple receivers.
    // In this case the expectation is that only the first shard will contain transactions and all
    // other shards will be empty.
    fn test_single_sender_txns() {
        let mut sender = generate_test_account();
        let mut receivers = Vec::new();
        let num_txns = 10;
        for _ in 0..num_txns {
            receivers.push(generate_test_account());
        }
        let transactions = create_signed_p2p_transaction(&mut sender, receivers);
        let partitioner = DependencyAwareUniformPartitioner {};
        let (accepted_txns, rejected_txns) = partitioner.partition(transactions.clone(), 4);
        // Create a map of transaction index to its expected status, first 3 transactions are expected to be accepted
        // and the rest are expected to be rejected.
        let mut expected_txn_statuses = HashMap::new();
        for index in 0..num_txns {
            if index < 3 {
                expected_txn_statuses.insert(index, PartitioningStatus::Accepted);
            } else {
                expected_txn_statuses.insert(index, PartitioningStatus::Discarded);
            }
        }
        verify_txn_statuses_and_shards(
            &transactions,
            &accepted_txns,
            &rejected_txns,
            &expected_txn_statuses,
            4,
        );
    }

    #[test]
    // Test that the partitioner works correctly for no conflict transactions. In this case, the
    // expectation is that all transactions will be accepted and the rejected transactions map will be empty.
    fn test_non_conflicting_txns() {
        let num_txns = 10;
        let mut transactions = Vec::new();
        for _ in 0..num_txns {
            transactions.push(create_non_conflicting_p2p_transaction())
        }

        let partitioner = DependencyAwareUniformPartitioner {};
        let (accepted_txns, rejected_txns) = partitioner.partition(transactions.clone(), 4);
        // Create a map of transaction index to its expected status, all transactions are expected to be accepted.
        let mut expected_txn_statuses = HashMap::new();
        for index in 0..transactions.len() {
            expected_txn_statuses.insert(index, PartitioningStatus::Accepted);
        }
        verify_txn_statuses_and_shards(
            &transactions,
            &accepted_txns,
            &rejected_txns,
            &expected_txn_statuses,
            4,
        );
    }

    #[test]
    // Test that the partitioner works for same sender transactions across shards and in this case,
    // the first transaction from the sender is expected to be accepted and the rest are expected to be rejected.
    // Following is the expected behavior: S1 [*, A1, A2] S2[*, A3, A4] S3[*, A5]
    // All transactions from sender A except A1, A2 are rejected.
    fn test_conflicting_sender_ordering() {
        let num_shards = 3;
        let mut conflicting_sender = generate_test_account();
        let mut conflicting_transactions = Vec::new();
        for _ in 0..5 {
            conflicting_transactions.push(
                create_signed_p2p_transaction(&mut conflicting_sender, vec![
                    generate_test_account(),
                ])
                .remove(0),
            );
        }
        let mut non_conflicting_transactions = Vec::new();
        for _ in 0..5 {
            non_conflicting_transactions.push(create_non_conflicting_p2p_transaction());
        }

        let mut transactions = Vec::new();
        let mut conflicting_txn_index = 0;
        let mut non_conflicting_txn_index = 0;
        transactions.push(non_conflicting_transactions[non_conflicting_txn_index].clone());
        non_conflicting_txn_index += 1;
        transactions.push(conflicting_transactions[conflicting_txn_index].clone());
        conflicting_txn_index += 1;
        transactions.push(conflicting_transactions[conflicting_txn_index].clone());
        conflicting_txn_index += 1;
        transactions.push(non_conflicting_transactions[non_conflicting_txn_index].clone());
        non_conflicting_txn_index += 1;
        transactions.push(conflicting_transactions[conflicting_txn_index].clone());
        conflicting_txn_index += 1;
        transactions.push(conflicting_transactions[conflicting_txn_index].clone());
        conflicting_txn_index += 1;
        transactions.push(non_conflicting_transactions[non_conflicting_txn_index].clone());
        transactions.push(conflicting_transactions[conflicting_txn_index].clone());

        let partitioner = DependencyAwareUniformPartitioner {};
        let (accepted_txns, rejected_txns) =
            partitioner.partition(transactions.clone(), num_shards);
        // Create a map of transaction index to its expected status, all transactions are expected to be accepted.
        let mut expected_txn_statuses = HashMap::new();
        expected_txn_statuses.insert(0, PartitioningStatus::Accepted);
        expected_txn_statuses.insert(1, PartitioningStatus::Accepted);
        expected_txn_statuses.insert(2, PartitioningStatus::Accepted);
        expected_txn_statuses.insert(3, PartitioningStatus::Accepted);
        expected_txn_statuses.insert(4, PartitioningStatus::Discarded);
        expected_txn_statuses.insert(5, PartitioningStatus::Discarded);
        expected_txn_statuses.insert(6, PartitioningStatus::Accepted);
        expected_txn_statuses.insert(7, PartitioningStatus::Discarded);
        verify_txn_statuses_and_shards(
            &transactions,
            &accepted_txns,
            &rejected_txns,
            &expected_txn_statuses,
            num_shards,
        );
    }

    #[test]
    // Generates a bunch of random transactions and ensures that after the partitioning, there is
    // no conflict across shards.
    fn test_no_conflict_across_shards() {
        let mut rng = OsRng;
        let max_accounts = 500;
        let max_txns = 2000;
        let max_num_shards = 64;
        let num_accounts = rng.gen_range(1, max_accounts);
        let mut accounts = Vec::new();
        for _ in 0..num_accounts {
            accounts.push(generate_test_account());
        }
        let num_txns = rng.gen_range(1, max_txns);
        let mut transactions = Vec::new();
        let num_shards = rng.gen_range(1, max_num_shards);

        for _ in 0..num_txns {
            // randomly select a sender and receiver from accounts
            let sender_index = rng.gen_range(0, num_accounts);
            let receiver_index = rng.gen_range(0, num_accounts);
            let receiver = accounts[receiver_index].clone();
            let sender = &mut accounts[sender_index];
            transactions.push(create_signed_p2p_transaction(sender, vec![receiver]).remove(0));
        }
        let partitioner = DependencyAwareUniformPartitioner {};
        let (accepted_txns, _) = partitioner.partition(transactions.clone(), num_shards);
        // Build a map of storage location to corresponding shards and ensure that no storage location is present in more than one shard.
        let mut storage_location_to_shard_map = HashMap::new();
        for shard_id in accepted_txns.keys() {
            let shard = accepted_txns.get(shard_id).unwrap();
            for txn in shard {
                let storage_locations = txn.read_hints().iter().chain(txn.write_hints().iter());
                for storage_location in storage_locations {
                    if storage_location_to_shard_map.contains_key(storage_location) {
                        assert_eq!(
                            storage_location_to_shard_map.get(storage_location).unwrap(),
                            &shard_id
                        );
                    } else {
                        storage_location_to_shard_map.insert(storage_location, shard_id);
                    }
                }
            }
        }
    }

    #[test]
    // Test that the partitioner works correctly when there are no transactions.
    // The expectation is that both the accepted and rejected transactions maps will be empty.
    fn test_no_transactions() {
        let transactions = vec![];
        let partitioner = DependencyAwareUniformPartitioner {};
        let (accepted_txns, rejected_txns) = partitioner.partition(transactions, 4);
        assert_eq!(accepted_txns.len(), 0);
        assert_eq!(rejected_txns.len(), 0);
    }

    #[test]
    // Test that the partitioner works correctly when the number of transactions is less than the number of shards.
    // In this case, all transactions should be accepted, and the rejected transactions map should be empty.
    fn test_less_transactions_than_shards() {
        let mut sender = generate_test_account();
        let receivers = vec![generate_test_account(), generate_test_account()];
        let transactions = create_signed_p2p_transaction(&mut sender, receivers);
        let partitioner = DependencyAwareUniformPartitioner {};
        let (accepted_txns, rejected_txns) = partitioner.partition(transactions, 4);
        assert_eq!(accepted_txns.len(), 1);
        assert_eq!(accepted_txns.get(&0).unwrap().len(), 1);
        assert_eq!(rejected_txns.len(), 1);
    }
}
