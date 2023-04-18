// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::config::{utils::get_config_name, Error, LoggerConfig, NodeConfig, RoleType};
use aptos_types::chain_id::ChainId;
use serde_yaml::Value;

// Useful optimizer constants
const OPTIMIZER_STRING: &str = "Optimizer";

/// A trait for optimizing node configs (and their sub-configs) by tweaking
/// config values based on node roles, chain IDs and compiler features.
///
/// Note: The config optimizer respects the following order precedence when
/// determining whether or not to optimize a value:
/// 1. If a config value has been set in the local config file, that value
///    should be used (and the optimizer should not override it).
/// 2. If a config value has not been set in the local config file, the
///    optimizer may set the value (but, it is not required to do so).
/// 3. Finally, if the config optimizer chooses not to set a value, the default
///    value is used (as defined in the default implementation).
pub trait ConfigOptimizer {
    /// Get the name of the optimizer (e.g., for logging)
    fn get_optimizer_name() -> String {
        let config_name = get_config_name::<Self>().to_string();
        config_name + OPTIMIZER_STRING
    }

    /// Optimize the config according to the given node role and chain ID
    fn optimize(
        _node_config: &mut NodeConfig,
        _node_config_yaml: &Value,
        _node_role: RoleType,
        _chain_id: ChainId,
    ) -> Result<(), Error> {
        unimplemented!("optimize() must be implemented for each optimizer!");
    }
}

impl ConfigOptimizer for NodeConfig {
    fn optimize(
        node_config: &mut NodeConfig,
        node_config_yaml: &Value,
        node_role: RoleType,
        chain_id: ChainId,
    ) -> Result<(), Error> {
        // Optimize all of the relevant sub-configs
        // optimize_failpoints_config(node_config, node_config_yaml, node_role, chain_id)?;
        //optimize_fullnode_network_configs(
        //    node_config,
        //    node_config_yaml,
        //    node_role,
        //    chain_id,
        //)?;
        //IndexerConfig::optimize(node_config, node_role, chain_id)?;
        //IndexerGrpcConfig::optimize(node_config, node_role, chain_id)?;
        //InspectionServiceConfig::optimize(node_config, node_role, chain_id)?;
        LoggerConfig::optimize(node_config, node_config_yaml, node_role, chain_id)?;
        //MempoolConfig::optimize(node_config, node_role, chain_id)?;
        //PeerMonitoringServiceConfig::optimize(node_config, node_role, chain_id)?;
        //StateSyncConfig::optimize(node_config, node_role, chain_id)?;
        //StorageConfig::optimize(node_config, node_role, chain_id)?;
        //optimize_validator_network_config(
        //    node_config,
        //    node_config_yaml,
        //    node_role,
        //    chain_id,
        //)?;

        Ok(()) // All optimizers have finished successfully
    }
}

/// Optimize the failpoints config according to the node role and chain ID
fn optimize_failpoints_config(
    _node_config: &mut NodeConfig,
    _node_config_yaml: &str,
    _node_role: RoleType,
    _chain_id: ChainId,
) -> Result<(), Error> {
    unimplemented!();
}

/// Optimize the fullnode network configs according to the node role and chain ID
fn optimize_fullnode_network_configs(
    _node_config: &mut NodeConfig,
    _node_config_yaml: &str,
    _node_role: RoleType,
    _chain_id: ChainId,
) -> Result<(), Error> {
    unimplemented!();
}

/// Optimize the validator network config according to the node role and chain ID
fn optimize_validator_network_config(
    _node_config: &mut NodeConfig,
    _node_config_yaml: &str,
    _node_role: RoleType,
    _chain_id: ChainId,
) -> Result<(), Error> {
    unimplemented!();
}
