// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::network_id::NetworkId;
use aptos_crypto::x25519;
use aptos_types::PeerId;
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

// All modules should be declared below
mod api_config;
mod base_config;
mod config_sanitizer;
mod consensus_config;
mod error;
mod execution_config;
mod identity_config;
mod indexer_config;
mod indexer_grpc_config;
mod inspection_service_config;
mod logger_config;
mod mempool_config;
mod network_config;
mod node_config;
mod node_config_loader;
mod quorum_store_config;
mod safety_rules_config;
mod secure_backend_config;
mod state_sync_config;
mod storage_config;
mod test_config;

// All public usage statements should be declared below
pub use api_config::*;
pub use base_config::*;
pub use consensus_config::*;
pub use error::*;
pub use execution_config::*;
pub use identity_config::*;
pub use indexer_config::*;
pub use indexer_grpc_config::*;
pub use inspection_service_config::*;
pub use logger_config::*;
pub use mempool_config::*;
pub use network_config::*;
pub use node_config::*;
pub use quorum_store_config::*;
pub use safety_rules_config::*;
pub use secure_backend_config::*;
pub use state_sync_config::*;
pub use storage_config::*;
pub use test_config::*;
