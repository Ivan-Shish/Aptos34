// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::sharded_block_executor::{
    block_partitioner::{BlockPartitioner, UniformPartitioner},
    executor_shard::ExecutorShard,
};
use aptos_state_view::StateView;
use aptos_types::transaction::{Transaction, TransactionOutput};
use move_core_types::vm_status::VMStatus;
use std::sync::{Arc, Mutex};

mod block_partitioner;
mod executor_shard;

/// A wrapper around sharded block executors that manages multiple shards and aggregates the results.
pub struct ShardedBlockExecutor {
    num_executor_shards: usize,
    partitioner: Arc<dyn BlockPartitioner>,
    sharded_executor_thread_pool: Arc<rayon::ThreadPool>,
    executor_shards: Vec<ExecutorShard>,
}

impl ShardedBlockExecutor {
    pub fn new(num_executor_shards: usize, num_threads_per_executor: Option<usize>) -> Self {
        assert!(num_executor_shards > 0, "num_executor_shards must be > 0");
        let num_threads_per_executor = num_threads_per_executor.unwrap_or_else(|| {
            (num_cpus::get() as f64 / num_executor_shards as f64).ceil() as usize
        });
        let sharded_executor_thread_pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_executor_shards)
                .build()
                .unwrap(),
        );
        let mut executor_shards = vec![];
        for i in 0..num_executor_shards {
            executor_shards.push(ExecutorShard::new(i, num_threads_per_executor));
        }
        Self {
            num_executor_shards,
            partitioner: Arc::new(UniformPartitioner {}),
            sharded_executor_thread_pool,
            executor_shards,
        }
    }

    /// Execute a block of transactions in parallel by splitting the block into num_remote_executors partitions and
    /// dispatching each partition to a remote executor shard.
    pub fn execute_block<S: StateView + Sync + Send + 'static>(
        &self,
        state_view: &S,
        block: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        let block_partitions = self.partitioner.partition(block, self.num_executor_shards);

        let results = Arc::new(Mutex::new(Vec::with_capacity(self.num_executor_shards)));
        // Create a result vector with a default value for each thread
        for _ in 0..self.num_executor_shards {
            results.lock().unwrap().push(Ok(vec![]));
        }

        self.sharded_executor_thread_pool.scope(|s| {
            for (i, transactions) in block_partitions.into_iter().enumerate() {
                let results_clone = results.clone();
                s.spawn(move |_| {
                    let result = self.executor_shards[i].execute_block(state_view, transactions);
                    results_clone.lock().unwrap()[i] = result;
                });
            }
        });

        let mut aggregated_results = vec![];
        for result in Arc::try_unwrap(results)
            .unwrap()
            .into_inner()
            .unwrap()
            .into_iter()
        {
            aggregated_results.extend(result?);
        }
        Ok(aggregated_results)
    }
}
