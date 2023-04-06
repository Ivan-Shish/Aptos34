use crate::{
    config::{
        node_config_loader,
        node_config_loader::{NodeConfigLoader, PersistableConfig},
        ApiConfig, BaseConfig, ConsensusConfig, Error, ExecutionConfig, IndexerConfig,
        IndexerGrpcConfig, InspectionServiceConfig, LoggerConfig, MempoolConfig, NetworkConfig,
        PeerMonitoringServiceConfig, RoleType, SafetyRulesTestConfig, StateSyncConfig,
        StorageConfig, TestConfig,
    },
    network_id::NetworkId,
};
use aptos_crypto::x25519;
use aptos_types::account_address::AccountAddress as PeerId;
use rand::{prelude::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// The node configuration defines the configuration for a single Aptos
/// node (i.e., validator or fullnode). It is composed of module
/// configurations for each of the modules that the node uses (e.g.,
/// the API, indexer, mempool, state sync, etc.).
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub base: BaseConfig,
    #[serde(default)]
    pub consensus: ConsensusConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
    #[serde(default)]
    pub failpoints: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub full_node_networks: Vec<NetworkConfig>,
    #[serde(default)]
    pub indexer: IndexerConfig,
    #[serde(default)]
    pub indexer_grpc: IndexerGrpcConfig,
    #[serde(default)]
    pub inspection_service: InspectionServiceConfig,
    #[serde(default)]
    pub logger: LoggerConfig,
    #[serde(default)]
    pub mempool: MempoolConfig,
    #[serde(default)]
    pub peer_monitoring_service: PeerMonitoringServiceConfig,
    #[serde(default)]
    pub state_sync: StateSyncConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub test: Option<TestConfig>,
    #[serde(default)]
    pub validator_network: Option<NetworkConfig>,
}

impl NodeConfig {
    /// Returns the data directory for this config
    pub fn get_data_dir(&self) -> &Path {
        &self.base.data_dir
    }

    /// Returns the working directory for this config (if set),
    /// otherwise, returns the data directory.
    pub fn get_working_dir(&self) -> &Path {
        match &self.base.working_dir {
            Some(working_dir) => working_dir,
            None => self.get_data_dir(),
        }
    }

    /// Sets the data directory for this config
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        // Set the base directory
        self.base.data_dir = data_dir.clone();

        // Update the data directory for consensus and storage
        self.consensus.set_data_dir(data_dir.clone());
        self.storage.set_data_dir(data_dir);
    }

    /// Loads the node config from the given path and performs several processing
    /// steps. Note: paths used in the node config are either absolute or
    /// relative to the config location.
    pub fn load<P: AsRef<Path>>(input_path: P) -> Result<Self, Error> {
        let node_config_loader = NodeConfigLoader::new(input_path)?;
        node_config_loader.load_and_process_node_config()
    }

    /// Returns the peer of the node based on the role. If the node
    /// is a validator, the validator network peer is returned. Otherwise,
    /// the public fullnode network peer is returned.
    pub fn get_peer_id(&self) -> Option<PeerId> {
        match self.base.role {
            RoleType::Validator => self.validator_network.as_ref().map(NetworkConfig::peer_id),
            RoleType::FullNode => self
                .full_node_networks
                .iter()
                .find(|config| config.network_id == NetworkId::Public)
                .map(NetworkConfig::peer_id),
        }
    }

    /// Returns the identity key of the node based on the role. If the node
    /// is a validator, the validator network identity key is returned. Otherwise,
    /// the public fullnode network identity key is returned.
    pub fn identity_key(&self) -> Option<x25519::PrivateKey> {
        match self.base.role {
            RoleType::Validator => self
                .validator_network
                .as_ref()
                .map(NetworkConfig::identity_key),
            RoleType::FullNode => self
                .full_node_networks
                .iter()
                .find(|config| config.network_id == NetworkId::Public)
                .map(NetworkConfig::identity_key),
        }
    }

    /// Saves the config to the given output filepath
    pub fn save<P: AsRef<Path>>(&mut self, output_path: P) -> Result<(), Error> {
        // Save the execution config
        let output_dir = RootPath::new(&output_path);
        self.execution.save(&output_dir)?;

        // Write the entire config to the output path. This must be called
        // last as calling save() on sub-configs may change their fields.
        self.save_config(&output_path)?;

        Ok(())
    }

    /// Randomizes the various ports of the node config
    pub fn randomize_ports(&mut self) {
        self.api.randomize_ports();
        self.inspection_service.randomize_ports();
        self.storage.randomize_ports();
        self.logger.disable_console();

        if let Some(network) = self.validator_network.as_mut() {
            network.listen_address = crate::utils::get_available_port_in_multiaddr(true);
        }

        for network in self.full_node_networks.iter_mut() {
            network.listen_address = crate::utils::get_available_port_in_multiaddr(true);
        }
    }

    /// Generates a random config for testing purposes
    pub fn generate_random_config() -> Self {
        let mut rng = StdRng::from_seed([0u8; 32]);
        Self::generate_random_config_with_template(&NodeConfig::default(), &mut rng)
    }

    /// Generates a random config using the given template and rng
    pub fn generate_random_config_with_template(template: &Self, rng: &mut StdRng) -> Self {
        let mut node_config = template.clone();
        node_config.randomize_internal(rng);
        node_config
    }

    /// Randomizes the internal fields of the config
    fn randomize_internal(&mut self, rng: &mut StdRng) {
        let mut test = TestConfig::new_with_temp_dir(None);

        if self.base.role == RoleType::Validator {
            test.random_account_key(rng);
            let peer_id = test.auth_key.unwrap().derived_address();

            if self.validator_network.is_none() {
                let network_config = NetworkConfig::network_with_id(NetworkId::Validator);
                self.validator_network = Some(network_config);
            }

            let validator_network = self.validator_network.as_mut().unwrap();
            validator_network.random_with_peer_id(rng, Some(peer_id));
            // We want to produce this key twice
            test.random_execution_key(rng);

            let mut safety_rules_test_config = SafetyRulesTestConfig::new(peer_id);
            safety_rules_test_config.random_consensus_key(rng);
            self.consensus.safety_rules.test = Some(safety_rules_test_config);
        } else {
            self.validator_network = None;
            if self.full_node_networks.is_empty() {
                let network_config = NetworkConfig::network_with_id(NetworkId::Public);
                self.full_node_networks.push(network_config);
            }
            for network in &mut self.full_node_networks {
                network.random(rng);
            }
        }
        self.set_data_dir(test.temp_dir().unwrap().to_path_buf());
        self.test = Some(test);
    }

    /// Returns a node config using the given serialized data
    fn default_config(serialized: &str, path: &'static str) -> Self {
        let mut node_config =
            Self::parse(serialized).unwrap_or_else(|error| panic!("Error in {}: {}", path, error));
        node_config_loader::validate_node_config(&mut node_config)
            .unwrap_or_else(|error| panic!("Failed to validate node config! Error: {}", error));
        node_config
    }

    /// Returns a default config for a public full node
    pub fn default_for_public_full_node() -> Self {
        let contents = std::include_str!("test_data/public_full_node.yaml");
        Self::default_config(contents, "default_for_public_full_node")
    }

    /// Returns a default config for a validator
    pub fn default_for_validator() -> Self {
        let contents = std::include_str!("test_data/validator.yaml");
        Self::default_config(contents, "default_for_validator")
    }

    /// Returns a default config for a validator full node
    pub fn default_for_validator_full_node() -> Self {
        let contents = std::include_str!("test_data/validator_full_node.yaml");
        Self::default_config(contents, "default_for_validator_full_node")
    }
}

#[derive(Debug)]
pub struct RootPath {
    root_path: PathBuf,
}

impl RootPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root_path = if let Some(parent) = path.as_ref().parent() {
            parent.to_path_buf()
        } else {
            PathBuf::from("")
        };

        Self { root_path }
    }

    /// This function assumes that the path is already a directory
    pub fn new_path<P: AsRef<Path>>(path: P) -> Self {
        let root_path = path.as_ref().to_path_buf();
        Self { root_path }
    }

    /// This adds a full path when loading / storing if one is not specified
    pub fn full_path(&self, file_path: &Path) -> PathBuf {
        if file_path.is_relative() {
            self.root_path.join(file_path)
        } else {
            file_path.to_path_buf()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::SafetyRulesConfig;

    #[test]
    fn verify_configs() {
        NodeConfig::default_for_public_full_node();
        NodeConfig::default_for_validator();
        NodeConfig::default_for_validator_full_node();

        let contents = std::include_str!("test_data/safety_rules.yaml");
        SafetyRulesConfig::parse(contents)
            .unwrap_or_else(|e| panic!("Error in safety_rules.yaml: {}", e));
    }

    #[test]
    fn validate_invalid_network_id() {
        let mut config = NodeConfig::default_for_public_full_node();
        let network = config.full_node_networks.iter_mut().next().unwrap();
        network.network_id = NetworkId::Validator;
        assert!(matches!(
            config.validate_network_configs(),
            Err(Error::InvariantViolation(_))
        ));
    }
}
