// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// This has all configuation info for the benchmarks.

// Universe config.
pub const NUM_TRANSACTIONS_IN_BLOCK: usize = 1000;
pub const NUM_ACCOUNTS: usize = 2;
pub const MIN_ACCOUNT_BALANCE: u64 = 10_000_000_000;
pub const MAX_ACCOUNT_BALANCE: u64 = 10_000_000_001;

// Amounts to transfer between each acount pair.
pub const MIN_TRANSFER: u64 = 1;
pub const MAX_TRANSFER: u64 = 1;

// Parallel execution config.
pub const CONCURRENCY_LEVEL: usize = 8;

// Have you changed peer_to_peer.rs to use the right transfer function? Look for BENCHMARK-CHANGE.

// Have you changed universe.rs to use aggregator or integer? Look for BENCHMARK-CHANGE.
