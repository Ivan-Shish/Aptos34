// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_indexer_grpc_cache_worker::Args as CacheWorkerArgs;
use aptos_indexer_grpc_data_service::Args as DataServiceArgs;
use aptos_indexer_grpc_file_store::Args as FileStoreArgs;
use aptos_indexer_grpc_utils::config::{
    IndexerGrpcConfig, IndexerGrpcFileStoreConfig, LocalFileStore,
};
use clap::Parser;
use std::fs;
use tempfile::TempDir;
use testcontainers::{clients, core::WaitFor, images::generic::GenericImage};

// setup for testcontainers
#[tokio::test]
pub async fn setup_indexer_grpc_all() {
    aptos_logger::Logger::init_for_testing();
    let docker = clients::Cli::default();

    let redis_wait_for = WaitFor::message_on_stdout("Ready to accept connections");
    let redis_image = GenericImage::new("redis", "latest").with_wait_for(redis_wait_for);
    let redis_container = docker.run(redis_image);
    let redis_ipv4_port = redis_container.get_host_port_ipv4(6379);

    let tmp_dir = TempDir::new().expect("Could not create temp dir");
    let tmp_dir_path_str = tmp_dir.path().as_os_str().to_str().unwrap();

    // create configs
    for (i, service_name) in vec!["cache_worker", "file_store", "data_service"]
        .iter()
        .enumerate()
    {
        let config = IndexerGrpcConfig {
            fullnode_grpc_address: Some("127.0.0.1:50051".to_string()),
            data_service_grpc_listen_address: Some("127.0.0.1:50052".to_string()),
            redis_address: format!("127.0.0.1:{}", redis_ipv4_port),
            file_store: IndexerGrpcFileStoreConfig::LocalFileStore(LocalFileStore {
                local_file_store_path: tmp_dir.path().to_path_buf(),
            }),
            health_check_port: 9090 + i as u16,
            whitelisted_auth_tokens: Some(vec!["dummytoken".to_string()]),
        };

        let config_path = tmp_dir
            .path()
            .join(format!("test_indexer_grpc_{}.yaml", service_name));
        fs::write(config_path, serde_yaml::to_string(&config).unwrap());
    }

    let runtime = aptos_runtimes::spawn_named_runtime("indexertest".to_string(), None);
    runtime.spawn(async move {
        let cache_worker = CacheWorkerArgs {
            config_path: format!("{}/{}", tmp_dir_path_str, "test_indexer_grpc_cache_worker.yaml")
        };
        cache_worker.execute().await;
    });

    runtime.spawn(async move {
        let file_store = FileStoreArgs {
            config_path: format!("{}/{}", tmp_dir_path_str, "test_indexer_grpc_file_store.yaml")
        };
        file_store.execute().await;
    });

    runtime.spawn(async move {
        let data_service = DataServiceArgs {
            config_path: format!("{}/{}", tmp_dir_path_str, "test_indexer_grpc_data_service.yaml")

        };
        data_service.execute().await;
    });
}
