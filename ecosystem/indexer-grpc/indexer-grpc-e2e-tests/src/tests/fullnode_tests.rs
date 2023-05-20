// Copyright © Aptos Foundation

use anyhow::{bail, Result};
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tracing::{error, info};

use aptos_indexer_grpc_cache_worker::IndexerGrpcCacheWorkerConfig;
use aptos_indexer_grpc_file_store::IndexerGrpcFileStoreWorkerConfig;
use aptos_indexer_grpc_server_framework::{
    run_server_with_config, setup_logging, setup_panic_handler, GenericConfig, RunnableConfig,
};
use aptos_indexer_grpc_utils::cache_operator::CacheOperator;
use aptos_indexer_grpc_utils::config::{IndexerGrpcFileStoreConfig, LocalFileStore};
use aptos_indexer_grpc_utils::file_store_operator::{FileStoreOperator, LocalFileStoreOperator};

static TESTNET_REST_API_URL: &str = "http://localhost:8080/v1";
static TESTNET_FULLNODE_GRPC_URL: &str = "localhost:50051";
static REDIS_PRIMARY_URL: &str = "localhost:6379";

async fn start_server<T: RunnableConfig>(
    server_config: T,
) -> Result<(u16, JoinHandle<Result<()>>)> {
    let health_check_port = aptos_config::utils::get_available_port();
    let config = GenericConfig {
        health_check_port,
        server_config,
    };
    let server_name = config.server_config.get_server_name();
    info!(
        "Starting server {} with healtheck port {}",
        server_name, health_check_port
    );
    let runtime_handle = tokio::runtime::Handle::current();

    // runs the component's run, but we need the server run
    let join_handle = runtime_handle.spawn(async move { run_server_with_config(config).await });
    let startup_timeout_secs = 30;
    for i in 0..startup_timeout_secs {
        match reqwest::get(format!("http://localhost:{}/metrics", health_check_port)).await {
            Ok(_) => break,
            Err(e) => {
                if i == startup_timeout_secs - 1 {
                    let msg = if join_handle.is_finished() {
                        format!("Server failed on startup: {:#?}", join_handle.await)
                    } else {
                        "Server was still starting up".to_string()
                    };
                    println!("waiting...");
                    bail!(
                        "Server didn't come up within given timeout: {:#?} {}",
                        e,
                        msg
                    );
                }
            },
        }
        if join_handle.is_finished() {
            bail!(
                "Server returned error while starting up: {:#?}",
                join_handle.await
            );
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Ok((health_check_port, join_handle))
}

// These tests expect that the local environment has a running fullnode
// This can be done by using the docker-compose
// We will then simulate chaos by using (1) docker exec (2) docker-compose scale <service>=<num_replicas>
#[tokio::test]
pub async fn verify_docker_compose_setup() {
    reqwest::get(TESTNET_REST_API_URL)
        .await
        .unwrap()
        .error_for_status()
        .unwrap(); // we just want a good status code
}

#[tokio::test]
async fn test_cache_operator() {
    // we're going to run both cache worker and file store worker in the same process
    // so we need to centrally set up logging and panic handler, whereas they are usually done in the same service
    setup_logging();
    setup_panic_handler();
    let tmp_dir = TempDir::new().expect("Could not create temp dir"); // start with a new file store each time
    let cache_worker_config = IndexerGrpcCacheWorkerConfig {
        server_name: "tcw-1".to_string(),
        fullnode_grpc_address: TESTNET_FULLNODE_GRPC_URL.to_string(),
        file_store_config: IndexerGrpcFileStoreConfig::LocalFileStore(LocalFileStore {
            local_file_store_path: tmp_dir.path().to_path_buf(),
        }),
        redis_main_instance_address: REDIS_PRIMARY_URL.to_string(),
    };

    let file_store_config = IndexerGrpcFileStoreWorkerConfig {
        server_name: "tfs-1".to_string(),
        redis_main_instance_address: REDIS_PRIMARY_URL.to_string(),
        file_store_config: IndexerGrpcFileStoreConfig::LocalFileStore(LocalFileStore {
            local_file_store_path: tmp_dir.path().to_path_buf(),
        }),
    };
    let (_cache_worker_port, _cache_worker_handle) =
        start_server::<IndexerGrpcCacheWorkerConfig>(cache_worker_config)
            .await
            .unwrap();

    let (_file_store_port, _file_store_handle) =
        start_server::<IndexerGrpcFileStoreWorkerConfig>(file_store_config)
            .await
            .expect("FAILED TO START SERVER!!!");

    // tokio::time::sleep(std::time::Duration::from_secs(100)).await; // the cache worker should panic in this time

    // _cache_worker_handle.

    // // wait a bit
    // info!("Wait a bit");
    // tokio::time::sleep(std::time::Duration::from_secs(100)).await; // the cache worker should panic in this time

    // inspect the files
    let file_store_operator = LocalFileStoreOperator::new(tmp_dir.path().to_path_buf());
    let file_store_metadata = file_store_operator.get_file_store_metadata().await;
    if let Some(metadata) = file_store_metadata {
        info!("[Indexer Cache] File store metadata: {:?}", metadata);
    } else {
        error!("[Indexer Cache] File store is empty. Exit after 1 minute.");
    }

    let conn = redis::Client::open(format!("redis://{}", REDIS_PRIMARY_URL.to_string()))
        .expect("Create redis client failed.")
        .get_async_connection()
        .await
        .expect("Create redis connection failed.");

    // check that the cache was written to
    let mut cache_operator = CacheOperator::new(conn);
    let chain_id = cache_operator.get_chain_id().await.unwrap();
    assert!(chain_id == 4);

    // cache is making progress
    let check_cache_secs = 30;
    let check_cache_frequency_secs = 5;
    let mut latest_version = 0;
    let mut new_latest_version;

    // check the latest processed version by the cache worker
    for i in 0..check_cache_secs {
        tokio::time::sleep(std::time::Duration::from_secs(check_cache_frequency_secs)).await;
        new_latest_version = cache_operator.get_latest_version().await.unwrap();
        info!(
            "Processed {} versions since last check {}s ago...",
            new_latest_version - latest_version,
            check_cache_frequency_secs
        );
        assert!(new_latest_version > latest_version);
        latest_version = new_latest_version;
    }
}
