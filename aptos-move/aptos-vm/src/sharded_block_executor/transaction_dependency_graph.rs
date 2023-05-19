// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0
use aptos_types::transaction::analyzed_transaction::{AnalyzedTransaction, StorageLocation};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Node<'a> {
    txn: &'a AnalyzedTransaction,
    index: usize,
}

impl<'a> Node<'a> {
    pub(crate) fn new(txn: &'a AnalyzedTransaction, index: usize) -> Self {
        Node { txn, index }
    }

    pub fn get_index(&self) -> usize {
        self.index
    }
}

pub struct DependencyGraph<'a> {
    adjacency_list: HashMap<Node<'a>, HashSet<Node<'a>>>,
    // The reverse adjacency list is used to quickly find the dependencies of a transaction.
    reverse_adjacency_list: HashMap<Node<'a>, HashSet<Node<'a>>>,
}

impl<'a> DependencyGraph<'a> {
    pub fn new() -> Self {
        DependencyGraph {
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, source: Node<'a>, destination: Node<'a>) {
        // Get or create the dependency set for the target transaction
        let dependencies = self
            .adjacency_list
            .entry(source)
            .or_insert_with(HashSet::new);

        let reverse_dependencies = self
            .reverse_adjacency_list
            .entry(destination)
            .or_insert_with(HashSet::new);

        // Add the source transaction to the dependency set
        dependencies.insert(destination);
        reverse_dependencies.insert(source);
    }

    pub fn get_dependent_nodes(&self, node: Node<'a>) -> Option<&'a HashSet<Node>> {
        self.reverse_adjacency_list.get(&node)
    }

    pub fn create_dependency_graph(
        analyzed_transactions: &[AnalyzedTransaction],
    ) -> DependencyGraph {
        let mut dependency_graph = DependencyGraph::new();

        let read_hint_index = Self::build_hint_index(analyzed_transactions, |txn| txn.read_hints());

        // build an index of the transactions to their indices
        let mut txn_index = HashMap::new();
        for (index, txn) in analyzed_transactions.iter().enumerate() {
            txn_index.insert(txn, index);
        }

        // Iterate through the analyzed transactions
        for analyzed_txn in analyzed_transactions.iter() {
            // Iterate through the write hints of the current transaction
            for write_hint in analyzed_txn.write_hints() {
                if let Some(transactions) = read_hint_index.get(write_hint) {
                    // Iterate through the transactions that read from the current write hint
                    for &dependent_txn in transactions {
                        if dependent_txn != analyzed_txn {
                            // Add the dependent transaction to the dependencies
                            dependency_graph.add_dependency(
                                Node::new(dependent_txn, *txn_index.get(dependent_txn).unwrap()),
                                Node::new(analyzed_txn, *txn_index.get(analyzed_txn).unwrap()),
                            );
                        }
                    }
                }
            }
        }

        dependency_graph
    }

    fn build_hint_index<F>(
        analyzed_transactions: &[AnalyzedTransaction],
        hint_selector: F,
    ) -> HashMap<&StorageLocation, HashSet<&AnalyzedTransaction>>
    where
        F: Fn(&AnalyzedTransaction) -> &[StorageLocation],
    {
        let mut index: HashMap<&StorageLocation, HashSet<&AnalyzedTransaction>> = HashMap::new();

        // Iterate through the analyzed transactions
        for analyzed_txn in analyzed_transactions {
            // Get the hints using the provided closure
            let hints = hint_selector(analyzed_txn);

            // Iterate through the hints
            for hint in hints {
                // Get or create the set of transactions associated with this hint
                let transactions = index.entry(hint).or_insert_with(HashSet::new);

                // Add the current transaction to the set
                transactions.insert(analyzed_txn);
            }
        }

        index
    }
}
