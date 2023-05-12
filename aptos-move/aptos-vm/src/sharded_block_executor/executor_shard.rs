// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0
use crate::block_executor::BlockAptosVM;
use aptos_logger::trace;
use aptos_state_view::StateView;
use aptos_types::transaction::{Transaction, TransactionOutput};
use move_core_types::vm_status::VMStatus;
use std::sync::Arc;

/// A remote block executor that receives transactions from a channel and executes them in parallel.
/// Currently it runs in the local machine and it will be further extended to run in a remote machine.
pub struct ExecutorShard {
    shard_id: usize,
    executor_thread_pool: Arc<rayon::ThreadPool>,
    num_executor_threads: usize,
}

impl ExecutorShard {
    pub fn new(shard_id: usize, concurrency_level: usize) -> Self {
        let executor_thread_pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(concurrency_level)
                .build()
                .unwrap(),
        );
        Self {
            shard_id,
            executor_thread_pool,
            num_executor_threads: concurrency_level,
        }
    }

    pub fn execute_block<S: StateView + Sync + Send + 'static>(
        &self,
        state_view: &S,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        let result = BlockAptosVM::execute_block(
            self.executor_thread_pool.clone(),
            transactions,
            state_view,
            self.num_executor_threads,
        );
        trace!("Executed block in executor shard {}", self.shard_id);
        result
    }
}
