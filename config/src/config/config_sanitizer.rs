// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::config::{ApiConfig, BaseConfig, ConsensusConfig, Error, ExecutionConfig, IndexerConfig, IndexerGrpcConfig, NetworkConfig, NodeConfig};
use aptos_types::chain_id::ChainId;
use cfg_if::cfg_if;
use std::collections::HashSet;

/// A trait for validating and processing a node config
pub trait ConfigSanitizer {
    /// Validates and processes the config using the given chain ID
    fn sanitize(_node_config: &mut NodeConfig, _chain_id: ChainId) -> Result<(), Error> {
        panic!("sanitize() is not implemented!");
    }
}

impl ConfigSanitizer for NodeConfig {
    fn sanitize(node_config: &mut NodeConfig, chain_id: ChainId) -> Result<(), Error> {
        // Sanitize the API config
        ApiConfig::sanitize(node_config, chain_id)?;

        // Sanitize the base config
        BaseConfig::sanitize(node_config, chain_id)?;

        // Sanitize the consensus config
        ConsensusConfig::sanitize(node_config, chain_id)?;

        // Sanitize the execution config
        ExecutionConfig::sanitize(node_config, chain_id)?;

        // Sanitize the failpoints config
        sanitize_failpoints_config(node_config, chain_id)?;

        // Sanitize the fullnode network config
        sanitize_fullnode_network_configs(node_config, chain_id)?;

        // Sanitize the indexer config
        IndexerConfig::sanitize(node_config, chain_id)?;

        // Sanitize the indexer grpc config
        IndexerGrpcConfig::sanitize(node_config, chain_id)?;

        NetworkConfig::sanitize(node_config, chain_id)?;

        /*

            // Validate the inspection service config
            validate_inspection_service_config(node_config, chain_id)?;

            // Validate the logger config
            validate_logger_config(node_config, chain_id)?;

            // Validate the mempool config
            validate_mempool_config(node_config, chain_id)?;

            // Validate the peer monitoring service config
            validate_peer_monitoring_service_config(node_config, chain_id)?;

            // Validate the state sync config
            validate_state_sync_config(node_config, chain_id)?;

            // Validate the storage config
            validate_storage_config(node_config, chain_id)?;

            // Validate the test config
            validate_test_config(node_config, chain_id)?;

            // Validate the validator network config
            validate_validator_network_config(node_config, chain_id)?;
        */

        Ok(())
    }
}

/// Validate and process the given failpoints config according to the chain ID
fn sanitize_failpoints_config(
    node_config: &mut NodeConfig,
    chain_id: ChainId,
) -> Result<(), Error> {
    // Check if failpoints are enabled
    let mut failpoints_enabled = false;
    cfg_if! {
        if #[cfg(feature = "failpoints")] {
            failpoints_enabled = true;
        }
    }

    // Verify that failpoints are not enabled in mainnet
    if chain_id.is_mainnet() && failpoints_enabled {
        return Err(Error::Validation(
            "Failpoints are not supported on mainnet nodes".into(),
        ));
    }

    // Ensure that the failpoints config is populated appropriately
    if let Some(failpoints) = &node_config.failpoints {
        if failpoints.is_empty() && failpoints_enabled {
            return Err(Error::Validation(
                "Failpoints are enabled, but the failpoints config is empty?".into(),
            ));
        } else if !failpoints.is_empty() && !failpoints_enabled {
            return Err(Error::Validation(
                "Failpoints are disabled, but the failpoints config is not empty!".into(),
            ));
        }
    } else {
        if failpoints_enabled {
            return Err(Error::Validation(
                "Failpoints are enabled, but the failpoints config is None!".into(),
            ));
        }
    }

    Ok(())
}

/// Ensures the given field value is not zero
pub fn verify_field_not_zero<T>(value: T, field_name: &str) -> Result<(), Error> {
    if value == 0 {
        Err(Error::Validation(format!(
            "The given field ({:?}) cannot be zero!",
            field_name
        )))
    } else {
        Ok(())
    }
}
