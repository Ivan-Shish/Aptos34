// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use aptos_indexer_grpc_cache_worker::Args;


#[tokio::main]
async fn main() {
    aptos_logger::Logger::new().init();
    aptos_crash_handler::setup_panic_handler();

    // load config and run
    Args::parse().execute().await;
}
