// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

pub mod metrics;
pub mod processor;


use clap::Parser;
use processor::Processor;
use aptos_indexer_grpc_utils::register_probes_and_metrics_handler;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Parser)]
pub struct Args {
    #[clap(short, long)]
    pub config_path: String,
}

impl Args {
    pub async fn execute(&self) { // TODO: give me a proper return type that can be used to exit
        let config = aptos_indexer_grpc_utils::config::IndexerGrpcConfig::load(
            std::path::PathBuf::from(&self.config_path),
        )
        .unwrap();

        let runtime = aptos_runtimes::spawn_named_runtime("indexerfile".to_string(), None);

        let health_port = config.health_check_port;
        runtime.spawn(async move {
            let mut processor = Processor::new(config);
            processor.run().await;
        });

        // Start liveness and readiness probes.
        runtime.spawn(async move {
            register_probes_and_metrics_handler(health_port).await;
        });

        let term = Arc::new(AtomicBool::new(false));
        while !term.load(Ordering::Acquire) {
            std::thread::park();
        }
    }
}
