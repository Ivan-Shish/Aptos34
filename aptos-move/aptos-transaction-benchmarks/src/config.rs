// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

// PAPER-BENCHMARK.

/// The number of accounts created by default.
pub const NUM_ACCOUNTS: usize = 2;

/// The number of transactions created by default.
pub const NUM_TRANSACTIONS: usize = 10000;

pub const CONCURRENCY_LEVEL: usize = 8;

pub const MIN_TRANSFER_AMOUNT: u64 = 1;
pub const MAX_TRANSFER_AMOUNT: u64 = 2;

pub const MIN_ACCOUNT_BALANCE: u64 = 10_000_000_000;
pub const MAX_ACCOUNT_BALANCE: u64 = 10_000_000_001;
