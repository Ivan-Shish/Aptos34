// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_api::Context;
use std::sync::Arc;

pub struct ApiV2Config {
    pub context: Arc<Context>,
}

impl ApiV2Config {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }
}
