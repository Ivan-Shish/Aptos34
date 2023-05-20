// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use aptos_config::{config::NodeConfig, utils::get_genesis_txn};
use aptos_db::AptosDB;
use aptos_executor::db_bootstrapper::maybe_bootstrap;
use aptos_logger::debug;
use aptos_storage_interface::{DbReader, DbReaderWriter};
use aptos_types::{
    chain_id::ChainId,
    epoch_state::EpochState,
    on_chain_config::{
        access_path_for_config, OnChainConfig, OnChainConfigPayload, ON_CHAIN_CONFIG_REGISTRY,
    },
    state_store::state_key::{StateKey, StateKeyInner},
    transaction::{Transaction, WriteSetPayload},
    waypoint::Waypoint,
    write_set::WriteOp,
};
use aptos_vm::AptosVM;
use std::{collections::HashMap, fs, net::SocketAddr, path::Path, sync::Arc, time::Instant};
use tokio::runtime::Runtime;

/// A simple struct that holds the original genesis state for
/// a node that has not yet been initialized and is attempting
/// to fast sync. The struct is useful for providing relevant
/// state from the genesis blob to various applications, so that
/// they do not need to rely on the genesis blob being in the db.
#[derive(Clone, Debug)]
pub struct GenesisState {
    chain_id: ChainId,
    epoch_state: Option<EpochState>,
    on_chain_config_payload: OnChainConfigPayload,
}

impl GenesisState {
    pub fn new(
        chain_id: ChainId,
        epoch_state: Option<EpochState>,
        on_chain_config_payload: OnChainConfigPayload,
    ) -> Self {
        Self {
            chain_id,
            epoch_state,
            on_chain_config_payload,
        }
    }

    /// Returns the chain id in the genesis state
    pub fn get_chain_id(&self) -> ChainId {
        self.chain_id
    }

    /// Returns the on-chain config payload in the genesis state
    pub fn get_on_chain_config_payload(&self) -> OnChainConfigPayload {
        self.on_chain_config_payload.clone()
    }
}

#[cfg(not(feature = "consensus-only-perf-test"))]
pub(crate) fn bootstrap_db(
    aptos_db: AptosDB,
    backup_service_address: SocketAddr,
) -> (Arc<AptosDB>, DbReaderWriter, Option<Runtime>) {
    use aptos_backup_service::start_backup_service;

    let (aptos_db, db_rw) = DbReaderWriter::wrap(aptos_db);
    let db_backup_service = start_backup_service(backup_service_address, aptos_db.clone());
    (aptos_db, db_rw, Some(db_backup_service))
}

/// In consensus-only mode, return a in-memory based [FakeAptosDB] and
/// do not run the backup service.
#[cfg(feature = "consensus-only-perf-test")]
pub(crate) fn bootstrap_db(
    aptos_db: AptosDB,
    _backup_service_address: SocketAddr,
) -> (
    Arc<aptos_db::fake_aptosdb::FakeAptosDB>,
    DbReaderWriter,
    Option<Runtime>,
) {
    use aptos_db::fake_aptosdb::FakeAptosDB;

    let (aptos_db, db_rw) = DbReaderWriter::wrap(FakeAptosDB::new(aptos_db));
    (aptos_db, None, db_rw, None)
}

/// Creates a RocksDb checkpoint for the consensus_db, state_sync_db,
/// ledger_db and state_merkle_db and saves it to the checkpoint_path.
/// Also, changes the working directory to run the node on the new path,
/// so that the existing data won't change. For now this is a test-only feature.
fn create_rocksdb_checkpoint_and_change_working_dir(
    node_config: &mut NodeConfig,
    working_dir: impl AsRef<Path>,
) {
    // Update the source and checkpoint directories
    let source_dir = node_config.storage.dir();
    node_config.set_data_dir(working_dir.as_ref().to_path_buf());
    let checkpoint_dir = node_config.storage.dir();
    assert!(source_dir != checkpoint_dir);

    // Create rocksdb checkpoint directory
    fs::create_dir_all(&checkpoint_dir).unwrap();

    // Open the database and create a checkpoint
    AptosDB::create_checkpoint(
        &source_dir,
        &checkpoint_dir,
        node_config
            .storage
            .rocksdb_configs
            .use_sharded_state_merkle_db,
    )
    .expect("AptosDB checkpoint creation failed.");

    // Create a consensus db checkpoint
    aptos_consensus::create_checkpoint(&source_dir, &checkpoint_dir)
        .expect("ConsensusDB checkpoint creation failed.");

    // Create a state sync db checkpoint
    let state_sync_db =
        aptos_state_sync_driver::metadata_storage::PersistentMetadataStorage::new(&source_dir);
    state_sync_db
        .create_checkpoint(&checkpoint_dir)
        .expect("StateSyncDB checkpoint creation failed.");
}

/// Applies the genesis transaction to storage (if required), or directly
/// extracts the genesis state from the transaction and returns it. This
/// is necessary for nodes that are fast syncing for the first time.
fn determine_genesis_state(
    node_config: &&mut NodeConfig,
    db_rw: &DbReaderWriter,
    genesis_transaction: &Transaction,
    genesis_waypoint: Waypoint,
) -> anyhow::Result<Option<GenesisState>> {
    // Check if the genesis transaction has already been applied to storage
    let genesis_already_applied = db_rw.reader.get_latest_transaction_info_option()?.is_some();

    // Determine if we're currently fast syncing
    let driver_config = node_config.state_sync.state_sync_driver;
    let fast_sync_mode = driver_config.bootstrapping_mode.is_fast_sync_mode();

    // If genesis hasn't been applied to the database yet, and we're fast syncing,
    // we should skip applying the genesis transaction and extract the state manually.
    if fast_sync_mode && !genesis_already_applied {
        debug!("Extracting the genesis state directly from the genesis transaction!");
        let genesis_state = extract_genesis_state(genesis_transaction)?;
        return Ok(Some(genesis_state));
    }

    // Otherwise, we should commit genesis to the database (if it
    // hasn't already been applied) and return None.
    maybe_bootstrap::<AptosVM>(db_rw, genesis_transaction, genesis_waypoint)
        .map_err(|err| anyhow!("DB failed to bootstrap {}", err))?;
    Ok(None)
}

/// Extracts the genesis state from the genesis transaction
fn extract_on_chain_config_payload(
    genesis_transaction: &Transaction,
) -> anyhow::Result<OnChainConfigPayload> {
    // Get the genesis change set from the transaction
    let genesis_change_set = match genesis_transaction {
        Transaction::GenesisTransaction(WriteSetPayload::Direct(change_set)) => change_set,
        _ => {
            return Err(anyhow!(
                "The genesis transaction has the incorrect type: {:?}!",
                genesis_transaction
            ));
        },
    };

    // Go through all on-chain configs and fetch the config data
    let mut on_chain_configs = HashMap::new();
    for config_id in ON_CHAIN_CONFIG_REGISTRY {
        // Get the config state key
        let config_access_path = access_path_for_config(*config_id)?;
        let config_state_key = StateKey::from(StateKeyInner::AccessPath(config_access_path));

        // Get the write op from the write set
        let write_set_mut = genesis_change_set.clone().write_set().clone().into_mut();
        let write_op = write_set_mut.get(&config_state_key).ok_or_else(|| {
            anyhow!(
                "The genesis transaction does not contain the write op for the config ID {:?}!",
                config_id
            )
        })?;

        // Extract the config bytes from the write op
        let write_op_bytes = match write_op {
            WriteOp::Creation(bytes) => bytes,
            WriteOp::Modification(bytes) => bytes,
            WriteOp::CreationWithMetadata { data, metadata: _ } => data,
            WriteOp::ModificationWithMetadata { data, metadata: _ } => data,
            _ => {
                return Err(anyhow!(
                "The genesis transaction does not contain the correct write op for the on-chain config!"
            ));
            },
        };

        // Save the config data
        on_chain_configs.insert(*config_id, write_op_bytes.to_vec());
    }

    // Create and return the on-chain config payload
    let genesis_epoch = 1; // TODO(joshlind): is this correct?
    Ok(OnChainConfigPayload::new(
        genesis_epoch,
        Arc::new(on_chain_configs),
    ))
}

/// Extracts the genesis state from the given genesis transaction
fn extract_genesis_state(genesis_transaction: &Transaction) -> anyhow::Result<GenesisState> {
    // Extract the on-chain config payload from the genesis transaction
    let on_chain_config_payload = extract_on_chain_config_payload(genesis_transaction)?;

    // Get the chain ID from the config payload
    let chain_id_bytes = on_chain_config_payload
        .configs()
        .get(&ChainId::CONFIG_ID)
        .ok_or_else(|| anyhow!("The on-chain config payload does not contain the chain ID!"))?;
    let chain_id = ChainId::deserialize_into_config(chain_id_bytes)
        .map_err(|error| anyhow!("Failed to deserialize the chain ID: {:?}", error))?;

    // Create and return the genesis state
    let genesis_state = GenesisState::new(chain_id, None, on_chain_config_payload);
    Ok(genesis_state)
}

/// Creates any rocksdb checkpoints, opens the storage database,
/// starts the backup service, handles genesis initialization and returns
/// the various handles.
pub fn initialize_database_and_checkpoints(
    node_config: &mut NodeConfig,
) -> anyhow::Result<(
    Arc<dyn DbReader>,
    Option<GenesisState>,
    DbReaderWriter,
    Option<Runtime>,
    Waypoint,
)> {
    // If required, create RocksDB checkpoints and change the working directory.
    // This is test-only.
    if let Some(working_dir) = node_config.base.working_dir.clone() {
        create_rocksdb_checkpoint_and_change_working_dir(node_config, working_dir);
    }

    // Open the database
    let instant = Instant::now();
    let aptos_db = AptosDB::open(
        &node_config.storage.dir(),
        false, /* readonly */
        node_config.storage.storage_pruner_config,
        node_config.storage.rocksdb_configs,
        node_config.storage.enable_indexer,
        node_config.storage.buffered_state_target_items,
        node_config.storage.max_num_nodes_per_lru_cache_shard,
    )
    .map_err(|err| anyhow!("DB failed to open {}", err))?;
    let (aptos_db, db_rw, backup_service) =
        bootstrap_db(aptos_db, node_config.storage.backup_service_address);

    // Get the genesis transaction and waypoint
    let genesis_transaction = get_genesis_txn(node_config).unwrap_or_else(|| {
        panic!("The genesis transaction is missing! Double check the genesis location in the node config!");
    });
    let genesis_waypoint = node_config.base.waypoint.genesis_waypoint();

    // Apply the genesis transaction to storage or extract the genesis state manually
    let genesis_state =
        determine_genesis_state(&node_config, &db_rw, genesis_transaction, genesis_waypoint)?;

    // Log the duration to open storage
    debug!(
        "Storage service started in {} ms",
        instant.elapsed().as_millis()
    );

    Ok((
        aptos_db,
        genesis_state,
        db_rw,
        backup_service,
        genesis_waypoint,
    ))
}
