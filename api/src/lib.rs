// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use poem_openapi::Tags;

mod accept_type;
pub mod accounts;
pub mod basic;
mod bcs_payload;
pub mod blocks;
pub mod check_size;
pub mod context;
pub mod error_converter;
pub mod events;
mod failpoint;
pub mod index;
pub mod log;
pub mod metrics;
pub mod page;
pub mod response;
pub mod set_failpoints;
pub mod state;
#[cfg(test)]
pub mod tests;
pub mod transactions;
pub mod view_function;

/// API categories for the OpenAPI spec
#[derive(Tags)]
pub enum ApiTags {
    /// Access to accounts, resources, and modules
    Accounts,
    /// Access to blocks
    Blocks,

    /// Access to events
    Events,

    /// Experimental APIs, no guarantees
    Experimental,

    /// General information
    General,

    /// Access to tables
    Tables,

    /// Access to transactions
    Transactions,

    /// View functions,
    View,
}

// Note: Many of these exports are just for the test-context crate, which is
// needed outside of the API, e.g. for fh-stream.
pub use context::Context;
pub use response::BasicError;
