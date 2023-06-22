// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::modules::Module;
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn modules(&self, _ctx: &Context<'_>, _module_ids: Vec<String>) -> Vec<Module> {
        vec![]
    }
}

#[allow(dead_code)]
pub type ApiV2Schema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
