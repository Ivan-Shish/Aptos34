// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{LoadDestination, NetworkLoadTest};
use aptos_forge::{NetworkContext, NetworkTest, Result, Test};
use aptos_logger::info;
use aptos_sdk::move_types::account_address::AccountAddress;
use tokio::runtime::Runtime;

/// A simple test that measures end-to-end performance when
/// multiple public fullnodes (PFNs) are running and transactions
/// are being submitted to those PFNs.
pub struct PFNPerformance;

impl Test for PFNPerformance {
    fn name(&self) -> &'static str {
        "PFNPerformance"
    }
}

impl NetworkTest for PFNPerformance {
    fn run(&self, ctx: &mut NetworkContext<'_>) -> Result<()> {
        <dyn NetworkLoadTest>::run(self, ctx)
    }
}

impl NetworkLoadTest for PFNPerformance {
    /// We must override the setup function to: (i) create PFNs in
    /// the swarm; and (ii) use those PFNs as the load destination.
    fn setup(&self, ctx: &mut NetworkContext) -> Result<LoadDestination> {
        // Identify the version and node config for the PFNs
        let swarm = ctx.swarm();
        let pfn_version = swarm.versions().max().unwrap();

        // Create the PFN swarm
        let num_pfns = 10;
        info!("Creating {} public fullnodes!", num_pfns);
        let runtime = Runtime::new().unwrap();
        let pfn_peer_ids: Vec<AccountAddress> = (0..num_pfns)
            .map(|_| {
                // Add the PFN to the swarm
                let pfn_config = swarm.get_default_pfn_node_config();
                let peer_id = runtime
                    .block_on(swarm.add_full_node(&pfn_version, pfn_config))
                    .unwrap();

                // Verify the PFN was added
                if swarm.full_node(peer_id).is_none() {
                    panic!(
                        "Failed to locate the PFN in the swarm! Peer ID: {:?}",
                        peer_id
                    );
                }

                // Save the PFNs peer ID
                info!("Created new PFN with peer ID: {:?}", peer_id);
                peer_id
            })
            .collect();

        // Use the PFNs as the load destination
        Ok(LoadDestination::Peers(pfn_peer_ids))
    }
}
