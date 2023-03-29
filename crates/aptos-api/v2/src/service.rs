// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{modules::get_account_modules, resources::get_resources};
use anyhow::{Context as AnyhowContext, Result};
use aptos_api::context::Context;
use aptos_logger::info;
use aptos_protos::api::v2::{
    api_v2_server::{ApiV2, ApiV2Server},
    GetAccountModulesRequest, GetAccountModulesResponse, GetResourcesRequest, GetResourcesResponse,
    FILE_DESCRIPTOR_SET as API_V2_FILE_DESCRIPTOR_SET,
};
use aptos_protos::transaction::testing1::v1::FILE_DESCRIPTOR_SET as TRANSACTION_V1_TESTING_FILE_DESCRIPTOR_SET;
use aptos_protos::util::timestamp::FILE_DESCRIPTOR_SET as UTIL_TIMESTAMP_FILE_DESCRIPTOR_SET;
use aptos_protos::google::api::v1::FILE_DESCRIPTOR_SET as GOOGLE_API_V1_FILE_DESCRIPTOR_SET;
use protoc_wkt::google::protobuf::{FILE_DESCRIPTOR_SET as GOOGLE_PROTOBUF_FILE_DESCRIPTOR_SET};
use poem::{endpoint::TowerCompatExt, IntoEndpoint, Route};
use std::sync::Arc;
//use sync_wrapper::SyncWrapper;
use tonic::{transport::Server, Request, Response, Status};

// todo
#[derive(Clone)]
pub struct ApiV2Service {
    pub context: Arc<Context>,
}

// TODO: Temporary until issues in build_api_v2_service below are solved.
pub fn build_api_v2_runtime(
    context: Arc<Context>,
    runtime: &tokio::runtime::Runtime,
) -> Result<()> {
    let service = ApiV2Service { context };

    let reflection_service = tonic_reflection::server::Builder::configure()
        // Note: It is critical that the file descriptor set is registered for every
        // file that the API proto depends on recursively. If you don't, compilation
        // will still succeed but reflection will fail at runtime.
        //
        // TODO: Add a test for this / something in build.rs, this is a big footgun.
        .register_encoded_file_descriptor_set(API_V2_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(TRANSACTION_V1_TESTING_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(UTIL_TIMESTAMP_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(GOOGLE_API_V1_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(GOOGLE_PROTOBUF_FILE_DESCRIPTOR_SET)
        .build()
        .context("Failed to build reflection service")?;

    // https://github.com/hyperium/tonic/issues/1323
    runtime.spawn(async move {
        let address = "0.0.0.0:50052".parse().unwrap();
        Server::builder()
            .add_service(reflection_service)
            .add_service(ApiV2Server::new(service))
            .serve(address)
            .await
            .expect("Failed to start API v2 server");
        info!(address = address, "[api-v2] Started GRPC server");
    });

    Ok(())
}

pub fn build_api_v2_service(context: Arc<Context>) -> Result<impl IntoEndpoint> {
    /*
    let service = ApiV2Service { context };

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .context("Failed to build reflection service")?;

    let tower_service = Server::builder()
        .add_service(ApiV2Server::new(service))
        .into_service();

    // https://github.com/tower-rs/tower/issues/691
    let tower_service = SyncWrapper::new(tower::util::BoxCloneService::new(tower_service));

    let tower_service = tower_service.compat();

    // https://github.com/poem-web/poem/issues/536
    // https://github.com/hyperium/tonic/issues/1322
    let routes = Route::new().nest("/", tower_service);
    */

    // TODO: Temporary
    let routes = Route::new();

    Ok(routes)
}

#[tonic::async_trait]
impl ApiV2 for ApiV2Service {
    async fn get_account_modules(
        &self,
        request: Request<GetAccountModulesRequest>,
    ) -> Result<Response<GetAccountModulesResponse>, Status> {
        get_account_modules(self, request).await
    }

    async fn get_resources(
        &self,
        request: Request<GetResourcesRequest>,
    ) -> Result<Response<GetResourcesResponse>, Status> {
        get_resources(self, request).await.map_err(|e| e.into())
    }
}
