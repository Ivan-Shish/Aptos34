// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::config::{
    config_sanitizer::ConfigSanitizer, node_config_loader, Error, NodeConfig, RoleType, RootPath,
    WaypointConfig,
};
use aptos_logger::warn;
use aptos_types::chain_id::ChainId;
use cfg_if::cfg_if;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// A simple node config loader that performs basic config
/// validation and processing.
pub struct NodeConfigLoader<P> {
    node_config_path: P,
}

impl<P: AsRef<Path>> NodeConfigLoader<P> {
    pub fn new(node_config_path: P) -> Self {
        Self { node_config_path }
    }

    /// Load the node config, validate the configuration options
    /// and process the config for the current environment.
    pub fn load_and_process_node_config(&self) -> Result<NodeConfig, Error> {
        // Load the node config from disk
        let mut node_config = NodeConfig::load_config(&self.node_config_path)?;

        // Load the execution config
        let input_dir = RootPath::new(&self.node_config_path);
        node_config.execution.load(&input_dir)?;

        // Get the chain_id for the node
        let chain_id = get_chain_id(&node_config)?;

        // Validate and process the node config
        NodeConfig::sanitize(&mut node_config, chain_id)?;

        // Update the data directory
        node_config.set_data_dir(node_config.get_data_dir().to_path_buf());
        Ok(node_config)
    }
}

/// Get the chain ID for the node
fn get_chain_id(_node_config: &NodeConfig) -> Result<ChainId, Error> {
    unimplemented!("TODO: complete me!!")
}

/// The interface for persistable configs
pub trait PersistableConfig: Serialize + DeserializeOwned {
    /// Loads and serializes the persistable config from the given path
    fn load_config<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        // Open the file and read it into a string
        let config_path_string = path.as_ref().to_str().unwrap().to_string();
        let mut file = File::open(&path).map_err(|error| {
            Error::Unexpected(format!(
                "Failed to open config file: {:?}. Error: {:?}",
                config_path_string, error
            ))
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|error| {
            Error::Unexpected(format!(
                "Failed to read the config file into a string: {:?}. Error: {:?}",
                config_path_string, error
            ))
        })?;

        // Parse the file string
        Self::parse(&contents)
    }

    /// Saves the persistable config to the given output file path
    fn save_config<P: AsRef<Path>>(&self, output_file: P) -> Result<(), Error> {
        // Serialize the config into a string
        let contents = serde_yaml::to_vec(&self)
            .map_err(|e| Error::Yaml(output_file.as_ref().to_str().unwrap().to_string(), e))?;

        // Create the output file and write the config string to it
        let mut file = File::create(output_file.as_ref())
            .map_err(|e| Error::IO(output_file.as_ref().to_str().unwrap().to_string(), e))?;
        file.write_all(&contents)
            .map_err(|e| Error::IO(output_file.as_ref().to_str().unwrap().to_string(), e))?;

        Ok(())
    }

    /// Parses the given string into a persistable config
    fn parse(serialized: &str) -> Result<Self, Error> {
        serde_yaml::from_str(serialized).map_err(|e| Error::Yaml("config".to_string(), e))
    }
}

impl<T: ?Sized> PersistableConfig for T where T: Serialize + DeserializeOwned {}
