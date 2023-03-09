// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    accept_type::AcceptType,
    context::Context,
    response::{BasicResponse, BasicResponseStatus, BasicResult},
    ApiTags,
};
use aptos_api_types::{IndexResponse, IndexResponseBcs};
use arc_swap::ArcSwap;
use poem_openapi::OpenApi;
use std::sync::Arc;

/// API for the index, to retrieve the ledger information
pub struct IndexApi {
    pub context: Arc<Context>,
    pub index_response_bcs: Arc<ArcSwap<Option<BasicResponse<IndexResponse>>>>,
    pub index_response_json: Arc<ArcSwap<Option<BasicResponse<IndexResponse>>>>,
}

#[OpenApi]
impl IndexApi {
    /// Get ledger info
    ///
    /// Get the latest ledger information, including data such as chain ID,
    /// role type, ledger versions, epoch, etc.
    #[oai(
        path = "/",
        method = "get",
        operation_id = "get_ledger_info",
        tag = "ApiTags::General"
    )]
    async fn get_ledger_info(&self, accept_type: AcceptType) -> BasicResult<IndexResponse> {
        // Look up the response in the cache
        match accept_type {
            AcceptType::Json => {
                if let Some(result) = self.index_response_json.load().as_ref() {
                    return BasicResult::Ok(result.clone());
                }
            },
            AcceptType::Bcs => {
                if let Some(result) = self.index_response_bcs.load().as_ref() {
                    return BasicResult::Ok(result.clone());
                }
            },
        }

        // Otherwise, compute the response, cache it and return it
        self.context
            .check_api_output_enabled("Get ledger info", &accept_type)?;
        let ledger_info = self.context.get_latest_ledger_info()?;

        let node_role = self.context.node_role();

        match accept_type {
            AcceptType::Json => {
                let index_response = IndexResponse::new(
                    ledger_info.clone(),
                    node_role,
                    Some(aptos_build_info::get_git_hash()),
                );
                let result = BasicResponse::try_from_json((
                    index_response,
                    &ledger_info,
                    BasicResponseStatus::Ok,
                ));

                if let Ok(result) = &result {
                    self.index_response_json
                        .swap(Arc::new(Some(result.clone())));
                }
                result
            },
            AcceptType::Bcs => {
                let index_response = IndexResponseBcs::new(ledger_info.clone(), node_role);
                let result = BasicResponse::try_from_bcs((
                    index_response,
                    &ledger_info,
                    BasicResponseStatus::Ok,
                ));

                if let Ok(result) = &result {
                    self.index_response_json
                        .swap(Arc::new(Some(result.clone())));
                }
                result
            },
        }
    }
}
