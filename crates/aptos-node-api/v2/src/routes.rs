// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{schema::QueryRoot, ApiV2Config};
use anyhow::Result;
use aptos_api_types::X_APTOS_CLIENT;
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_poem::GraphQL;
use poem::{
    get, handler,
    http::{header, Method},
    middleware::Cors,
    web::Html,
    EndpointExt, IntoEndpoint, IntoResponse, Route,
};

#[handler]
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/v2").finish())
}

pub fn build_api_v2_routes(config: ApiV2Config) -> Result<impl IntoEndpoint> {
    // Build the Schema. We attach the DB context to it.
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(config.context.clone())
        .finish();

    let cors = Cors::new()
        // To allow browsers to use cookies (for cookie-based sticky routing in the LB)
        // we must enable this: https://stackoverflow.com/a/24689738/3846032
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![
            header::HeaderName::from_static(X_APTOS_CLIENT),
            header::CONTENT_TYPE,
            header::ACCEPT,
        ]);

    // Build routes for the API
    let routes = Route::new()
        .at("/", get(graphiql).post(GraphQL::new(schema)))
        .with(cors);

    Ok(routes)
}
