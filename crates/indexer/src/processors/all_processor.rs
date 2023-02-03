// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    database::PgDbPool,
    indexer::{
        errors::TransactionProcessingError, processing_result::ProcessingResult,
        transaction_processor::TransactionProcessor,
    },
};
use aptos_api_types::Transaction;
use async_trait::async_trait;
use rayon::prelude::*;
use std::{fmt::Debug, sync::Arc};

pub const NAME: &str = "all_processor";
pub struct AllTransactionProcessor {
    connection_pool: PgDbPool,
    processors: Vec<Arc<dyn TransactionProcessor>>,
}

impl AllTransactionProcessor {
    pub fn new(connection_pool: PgDbPool, processors: Vec<Arc<dyn TransactionProcessor>>) -> Self {
        Self {
            connection_pool,
            processors,
        }
    }
}

impl Debug for AllTransactionProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = &self.connection_pool.state();
        write!(
            f,
            "AllTransactionProcessor {{ connections: {:?}  idle_connections: {:?} }}",
            state.connections, state.idle_connections
        )
    }
}

#[async_trait]
impl TransactionProcessor for AllTransactionProcessor {
    fn name(&self) -> &'static str {
        NAME
    }

    async fn process_transactions(
        &self,
        transactions: &[Transaction],
        start_version: u64,
        end_version: u64,
    ) -> Result<ProcessingResult, TransactionProcessingError> {
        let futs = self
            .processors
            .par_iter()
            .map(|processor| {
                processor.process_transactions(&transactions, start_version, end_version)
            })
            .collect::<Vec<_>>();
        let results = futures::future::join_all(futs).await;
        let error_result = results
            .iter()
            .find(|result| result.as_ref().err().is_some());
        if error_result.is_some() {
            Err(TransactionProcessingError::TransactionCommitError((
                anyhow::Error::msg("all_processor"),
                start_version,
                end_version,
                self.name(),
            )))
        } else {
            Ok(ProcessingResult::new(
                self.name(),
                start_version,
                end_version,
            ))
        }
    }

    fn connection_pool(&self) -> &PgDbPool {
        &self.connection_pool
    }
}
