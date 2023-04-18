// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{
        config_optimizer::ConfigOptimizer, config_sanitizer::ConfigSanitizer, utils::RootPath,
        Error, NodeConfig, PersistableConfig,
    },
    utils::get_genesis_txn,
};
use aptos_types::{
    chain_id::ChainId,
    on_chain_config::OnChainConfig,
    state_store::state_key::{StateKey, StateKeyInner},
    transaction::{Transaction, WriteSetPayload},
    write_set::WriteOp,
};
use serde_yaml::Value;
use std::path::Path;

/// A simple node config loader that performs basic config
/// sanitization and post-processing.
pub struct NodeConfigLoader<P> {
    node_config_path: P,
}

impl<P: AsRef<Path>> NodeConfigLoader<P> {
    pub fn new(node_config_path: P) -> Self {
        Self { node_config_path }
    }

    /// Load the node config, validate the configuration options
    /// and process the config for the current environment.
    pub fn load_and_sanitize_config(&self) -> Result<NodeConfig, Error> {
        // Load the node config from disk
        let mut node_config = NodeConfig::load_config(&self.node_config_path)?;

        // Load the execution config
        let input_dir = RootPath::new(&self.node_config_path);
        node_config.execution.load_from_path(&input_dir)?;

        // Optimize and sanitize the node config
        let node_config_yaml = get_node_config_yaml(&self.node_config_path)?;
        optimize_and_sanitize_node_config(&mut node_config, node_config_yaml)?;

        // Update the data directory
        node_config.set_data_dir(node_config.get_data_dir().to_path_buf());
        Ok(node_config)
    }
}

/// Return the node config file contents as a string
fn get_node_config_yaml<P: AsRef<Path>>(node_config_path: P) -> Result<Value, Error> {
    // Read the file contents into a string
    let node_config_yaml = NodeConfig::read_config_file(&node_config_path)?;

    // Parse the file contents as a yaml value
    let node_config_yaml = serde_yaml::from_str(&node_config_yaml).map_err(|error| {
        Error::Yaml(
            "Failed to parse the node config file into a YAML value".into(),
            error,
        )
    })?;

    Ok(node_config_yaml)
}

/// Optimize and sanitize the node config for the current environment
fn optimize_and_sanitize_node_config(
    node_config: &mut NodeConfig,
    node_config_yaml: Value,
) -> Result<(), Error> {
    // Get the role and chain_id for the node
    let node_role = node_config.base.role;
    let chain_id = match get_chain_id(node_config) {
        Ok(chain_id) => chain_id,
        Err(error) => {
            println!("Failed to get the chain ID from the genesis blob! Skipping config sanitization. Error: {:?}", error);
            return Ok(());
        },
    };

    // Optimize the node config
    NodeConfig::optimize(node_config, &node_config_yaml, node_role, chain_id)?;

    // Sanitize the node config
    NodeConfig::sanitize(node_config, node_role, chain_id)
}

/// Get the chain ID for the node
fn get_chain_id(node_config: &NodeConfig) -> Result<ChainId, Error> {
    // TODO: can we make this less hacky?

    // Load the genesis transaction from disk
    let genesis_txn = get_genesis_txn(node_config).ok_or(Error::InvariantViolation(
        "The genesis transaction was not found!".to_string(),
    ))?;

    // Extract the chain ID from the genesis transaction
    match genesis_txn {
        Transaction::GenesisTransaction(WriteSetPayload::Direct(change_set)) => {
            // Get the chain ID state key
            let chain_id_access_path = ChainId::access_path().map_err(|error| {
                Error::InvariantViolation(format!(
                    "Failed to get the chain ID access path! Error: {:?}",
                    error
                ))
            })?;
            let chain_id_state_key =
                StateKey::from(StateKeyInner::AccessPath(chain_id_access_path));

            // Get the write op from the write set
            let write_set_mut = change_set.clone().write_set().clone().into_mut();
            let write_op = write_set_mut.get(&chain_id_state_key).ok_or_else(|| {
                Error::InvariantViolation(
                    "The genesis transaction does not contain the write op for the chain id!"
                        .into(),
                )
            })?;

            // Extract the chain ID from the write op
            let write_op_bytes = match write_op {
                WriteOp::Creation(bytes) => bytes,
                WriteOp::Modification(bytes) => bytes,
                WriteOp::CreationWithMetadata { data, metadata: _ } => data,
                WriteOp::ModificationWithMetadata { data, metadata: _ } => data,
                _ => {
                    return Err(Error::InvariantViolation(
                        "The genesis transaction does not contain the correct write op for the chain ID!".into(),
                    ));
                },
            };
            let chain_id = ChainId::deserialize_into_config(write_op_bytes).map_err(|error| {
                Error::InvariantViolation(format!(
                    "Failed to deserialize the chain ID: {:?}",
                    error
                ))
            })?;

            Ok(chain_id)
        },
        _ => Err(Error::InvariantViolation(format!(
            "The genesis transaction has the incorrect type: {:?}!",
            genesis_txn
        ))),
    }
}
