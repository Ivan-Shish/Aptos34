// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use cfg_block::cfg_block;
use serde::{Deserialize, Serialize};

// Useful performance monitoring constants
const DIRECT_SEND_DATA_SIZE: u64 = 512 * 1024; // 512KB
const DIRECT_SEND_INTERVAL_USEC: u64 = 1000; // 1 ms
const RPC_DATA_SIZE: u64 = 512 * 1024; // 512KB
const RPC_INTERVAL_USEC: u64 = 1000; // 1 ms
const RPC_TIMEOUT_MS: u64 = 10_000; // 10 seconds

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PeerMonitoringServiceConfig {
    pub enable_peer_monitoring_client: bool, // Whether or not to spawn the monitoring client
    pub latency_monitoring: LatencyMonitoringConfig,
    pub max_concurrent_requests: u64, // Max num of concurrent server tasks
    pub max_network_channel_size: u64, // Max num of pending network messages
    pub max_request_jitter_ms: u64, // Max amount of jitter (ms) that a request will be delayed for
    pub metadata_update_interval_ms: u64, // The interval (ms) between metadata updates
    pub network_monitoring: NetworkMonitoringConfig,
    pub node_monitoring: NodeMonitoringConfig,
    pub peer_monitor_interval_ms: u64, // The interval (ms) between peer monitor executions
    pub performance_monitoring: PerformanceMonitoringConfig,
}

impl Default for PeerMonitoringServiceConfig {
    fn default() -> Self {
        Self {
            enable_peer_monitoring_client: false,
            latency_monitoring: LatencyMonitoringConfig::default(),
            max_concurrent_requests: 1000,
            max_network_channel_size: 1000,
            max_request_jitter_ms: 1000, // Monitoring requests are very infrequent
            metadata_update_interval_ms: 5000,
            network_monitoring: NetworkMonitoringConfig::default(),
            node_monitoring: NodeMonitoringConfig::default(),
            peer_monitor_interval_ms: 1000,
            performance_monitoring: PerformanceMonitoringConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatencyMonitoringConfig {
    pub latency_ping_interval_ms: u64, // The interval (ms) between latency pings for each peer
    pub latency_ping_timeout_ms: u64,  // The timeout (ms) for each latency ping
    pub max_latency_ping_failures: u64, // Max ping failures before the peer connection fails
    pub max_num_latency_pings_to_retain: usize, // The max latency pings to retain per peer
}

impl Default for LatencyMonitoringConfig {
    fn default() -> Self {
        Self {
            latency_ping_interval_ms: 30_000, // 30 seconds
            latency_ping_timeout_ms: 20_000,  // 20 seconds
            max_latency_ping_failures: 3,
            max_num_latency_pings_to_retain: 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkMonitoringConfig {
    pub network_info_request_interval_ms: u64, // The interval (ms) between network info requests
    pub network_info_request_timeout_ms: u64,  // The timeout (ms) for each network info request
}

impl Default for NetworkMonitoringConfig {
    fn default() -> Self {
        Self {
            network_info_request_interval_ms: 60_000, // 1 minute
            network_info_request_timeout_ms: 10_000,  // 10 seconds
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NodeMonitoringConfig {
    pub node_info_request_interval_ms: u64, // The interval (ms) between node info requests
    pub node_info_request_timeout_ms: u64,  // The timeout (ms) for each node info request
}

impl Default for NodeMonitoringConfig {
    fn default() -> Self {
        Self {
            node_info_request_interval_ms: 20_000, // 20 seconds
            node_info_request_timeout_ms: 10_000,  // 10 seconds
        }
    }
}

// Note: to enable performance monitoring, the compilation feature "network-perf-test" is required.
// Simply enabling the config values here will not enable performance monitoring.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PerformanceMonitoringConfig {
    pub enable_direct_send_testing: bool, // Whether or not to enable direct send test mode
    pub direct_send_data_size: u64,       // The amount of data to send in each request
    pub direct_send_interval_usec: u64,   // The interval (microseconds) between requests
    pub enable_rpc_testing: bool,         // Whether or not to enable RPC test mode
    pub rpc_data_size: u64,               // The amount of data to send in each RPC request
    pub rpc_interval_usec: u64,           // The interval (microseconds) between RPC requests
    pub rpc_timeout_ms: u64,              // The timeout (ms) for each RPC request
}

cfg_block! {
    if #[cfg(feature = "network-perf-test")] { // Disabled by default
        impl Default for PerformanceMonitoringConfig {
            fn default() -> Self {
                Self {
                    enable_direct_send_testing: false, // Disabled by default
                    direct_send_data_size: DIRECT_SEND_DATA_SIZE,
                    direct_send_interval_usec: DIRECT_SEND_INTERVAL_USEC,
                    enable_rpc_testing: true, // Enable RPC testing by default
                    rpc_data_size: RPC_DATA_SIZE,
                    rpc_interval_usec: RPC_INTERVAL_USEC,
                    rpc_timeout_ms: RPC_TIMEOUT_MS,
                }
            }
        }
    } else {
        impl Default for PerformanceMonitoringConfig {
            fn default() -> Self {
                Self {
                    enable_direct_send_testing: false, // Disabled by default
                    direct_send_data_size: DIRECT_SEND_DATA_SIZE,
                    direct_send_interval_usec: DIRECT_SEND_INTERVAL_USEC,
                    enable_rpc_testing: false, // Disabled by default
                    rpc_data_size: RPC_DATA_SIZE,
                    rpc_interval_usec: RPC_INTERVAL_USEC,
                    rpc_timeout_ms: RPC_TIMEOUT_MS,
                }
            }
        }
    }
}
