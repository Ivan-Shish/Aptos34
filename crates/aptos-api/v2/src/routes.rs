// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{service::build_api_v2_service, ApiV2Config};
use anyhow::{Context as AnyhowContext, Result};
use poem::{
    http::{header, Method},
    middleware::Cors,
    EndpointExt, IntoEndpoint, Route,
};

// TODO: It'd be better if this returned just a Tower service.

/// Returns address it is running at.
pub fn build_api_v2_routes(config: ApiV2Config) -> Result<impl IntoEndpoint> {
    let api_service =
        build_api_v2_service(config.context.clone()).context("Failed to build API V2 service")?;

    let cors = Cors::new()
        // To allow browsers to use cookies (for cookie-based sticky
        // routing in the LB) we must enable this:
        // https://stackoverflow.com/a/24689738/3846032
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![header::CONTENT_TYPE, header::ACCEPT]);

    // Build routes for the API
    let routes = Route::new().nest("/", api_service).with(cors);

    Ok(routes)
}
