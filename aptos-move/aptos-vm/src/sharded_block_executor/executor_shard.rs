// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0
use crate::{block_executor::BlockAptosVM, sharded_block_executor::ExecutorShardCommand};
use aptos_logger::trace;
use aptos_types::transaction::TransactionOutput;
use move_core_types::vm_status::VMStatus;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

/// A remote block executor that receives transactions from a channel and executes them in parallel.
/// Currently it runs in the local machine and it will be further extended to run in a remote machine.
pub struct ExecutorShard<'a> {
    shard_id: usize,
    executor_thread_pool: Arc<rayon::ThreadPool>,
    num_executor_threads: usize,
    command_rx: Receiver<ExecutorShardCommand>,
    result_tx: Sender<Result<Vec<TransactionOutput>, VMStatus>>,
}

impl ExecutorShard<'_> {
    pub fn new(
        shard_id: usize,
        concurrency_level: usize,
        command_rx: Receiver<ExecutorShardCommand>,
        result_tx: Sender<Result<Vec<TransactionOutput>, VMStatus>>,
    ) -> Self {
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
            command_rx,
            result_tx,
        }
    }

    pub fn start(&self) {
        loop {
            let command = self.command_rx.recv().unwrap();
            match command {
                ExecutorShardCommand::ExecuteBlock(state_view, transactions) => {
                    trace!(
                        "Shard {} received ExecuteBlock command of block size {} ",
                        self.shard_id,
                        transactions.len()
                    );
                    let ret = BlockAptosVM::execute_block(
                        self.executor_thread_pool.clone(),
                        transactions,
                        state_view,
                        self.num_executor_threads,
                    );
                    self.result_tx.send(ret).unwrap();
                },
                ExecutorShardCommand::Stop => {
                    break;
                },
            }
        }
        trace!("Shard {} is shutting down", self.shard_id);
    }
}
