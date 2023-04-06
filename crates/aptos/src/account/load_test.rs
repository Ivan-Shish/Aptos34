// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::common::types::{CliCommand, CliError, CliTypedResult, TransactionOptions};
use aptos_rest_client::aptos_api_types::HashValue;
use aptos_types::transaction::{Script, TransactionPayload};
use async_trait::async_trait;
use clap::Parser;
use std::path::PathBuf;

// TODO: Add ability to transfer non-APT coins
// TODO: Add ability to not create account by default
/// Transfer APT between accounts
///
#[derive(Debug, Parser)]
pub struct LoadTest {
    #[clap(flatten)]
    pub(crate) txn_options: TransactionOptions,

    /// Number of transactions to send
    #[clap(long)]
    pub amount: u32,

    #[clap(long, parse(from_os_str))]
    pub compiled_script_path: PathBuf,
}

#[async_trait]
impl CliCommand<Vec<HashValue>> for LoadTest {
    fn command_name(&self) -> &'static str {
        "LoadTest"
    }

    async fn execute(self) -> CliTypedResult<Vec<HashValue>> {
        let bytes = std::fs::read(self.compiled_script_path.as_path()).map_err(|e| {
            CliError::IO(format!("Unable to read {:?}", self.compiled_script_path), e)
        })?;
        let payload = TransactionPayload::Script(Script::new(bytes, vec![], vec![]));

        self.txn_options
            .submit_many_transactions(payload, self.amount)
            .await
    }
}
