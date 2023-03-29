// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ApiError, service::ApiV2Service};
use anyhow::{Context as AnyhowContext, Result};
use aptos_api::{context::Context, page::determine_limit, response::BasicErrorWith404};
use aptos_api_types::{
    verify_module_identifier, Address, AptosErrorCode, AsConverter, IdentifierWrapper, MoveModule,
    MoveModuleBytecode, MoveResource, MoveStructTag, MoveValue, RawTableItemRequest,
    StateKeyWrapper, TableItemRequest, VerifyInput, VerifyInputWithRecursion, U64,
};
use aptos_indexer_grpc_fullnode::convert::convert_move_module_bytecode;
use aptos_logger::info;
use aptos_protos::api::v2::{
    api_v2_server::{ApiV2, ApiV2Server},
    move_module_wrapper, resource_wrapper, GetResourcesRequest, GetResourcesResponse,
    MoveModuleWrapper, ResourceWrapper, Resources, FILE_DESCRIPTOR_SET,
};
use aptos_state_view::TStateView;
use aptos_storage_interface::state_view::DbStateView;
use aptos_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    state_store::{
        state_key::{StateKey, StateKeyInner},
        table::TableHandle,
    },
    PeerId,
};
use aptos_vm::data_cache::AsMoveResolver;
use futures::future::join_all;
use itertools::Itertools;
use move_core_types::{
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
    move_resource::MoveStructType,
    resolver::ResourceResolver,
};
use poem::{endpoint::TowerCompatExt, IntoEndpoint, Route};
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryInto, str::FromStr, sync::Arc};
use tonic::{transport::Server, Code, Request, Response, Status};

// https://github.com/hyperium/tonic/discussions/1324

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
enum GetResourcesPageToken {
    // When getting all resource types, we return an index indicating which address
    // we should start at on the next call, along with perhaps a cursor if we were
    // part way through fetching the resources for that address.
    AllResourceTypes((u32, Option<StateKeyInner>)),
    // When getting just some resource types, we return an index indicating which
    // address + resource we should start at on the next call. So if the request
    // was for 5 addresses and 4 resources types, 7 would mean the 3rd resource in
    // the list for the 2nd address.
    SpecificResourceTypes(u32),
}

// advantages:
// better errors, no awkward mapping to status codes
// no U64 stuff, the types are real
// the cursor for pagination is returned as is
pub async fn get_resources(
    service: &ApiV2Service,
    request: Request<GetResourcesRequest>,
) -> Result<Response<GetResourcesResponse>, ApiError> {
    println!("Received request: {:?}", request);
    let request_inner = request.into_inner();
    let (ledger_info, ledger_version, state_view) = service
        .context
        .state_view::<BasicErrorWith404>(request_inner.ledger_version)
        .unwrap();
    let state_view = Arc::new(state_view);

    // Note: Addresses isn't just for accounts, but objects.
    let mut addresses = Vec::new();
    for address in request_inner.addresses {
        let address = AccountAddress::from_hex_literal(&address)
            .map_err(|e| ApiError::new(Code::InvalidArgument, e.to_string()))?;
        addresses.push(address);
    }

    let max_page_size = service.context.max_account_resources_page_size();

    // Confirm that at least one address was provided.
    if addresses.is_empty() {
        return Err(ApiError::new(
            Code::InvalidArgument,
            "At least one account address must be provided",
        ));
    }

    // Deserialize the page token if provided.
    let page_token = request_inner.page_token.map(|p| {
        bcs::from_bytes::<GetResourcesPageToken>(&p)
            .map_err(|e| ApiError::new(Code::InvalidArgument, e.to_string()))
            .unwrap()
    });

    let limit = determine_limit::<BasicErrorWith404>(
        request_inner.page_size.map(|p| p as u16),
        max_page_size,
        max_page_size,
        &ledger_info,
    )
    .map_err(|e| ApiError::new(Code::InvalidArgument, e.to_string()))?;

    // This is a map of address to resources. Resources is then a map of canonical
    // resource tag representation to resource.
    let mut raw_out: HashMap<AccountAddress, HashMap<StructTag, Vec<u8>>> = HashMap::new();

    let cursor = if request_inner.resource_types.is_empty() {
        let token = page_token
            .map(|t| match t {
                GetResourcesPageToken::AllResourceTypes(t) => Ok(t),
                GetResourcesPageToken::SpecificResourceTypes(_) => Err(ApiError::new(
                    Code::InvalidArgument,
                    "Unexpected token type",
                )),
            })
            .transpose()?;
        let mut address_index = token.as_ref().map(|t| t.0).unwrap_or(0);
        println!("Address index: {}", address_index);
        let mut cursor = token.and_then(|t| t.1.map(|i| StateKey::new(i)));

        // Get addresses starting from the index where we're meant to start.
        let addresses: Vec<AccountAddress> = addresses.drain(address_index as usize..).collect();

        let mut remaining_to_get = limit;
        let len = addresses.len();
        for address in addresses.into_iter() {
            println!(
                "Getting resources for address: {} {}",
                address, remaining_to_get
            );
            let (resources, next_state_key) = service
                .context
                .get_resources_by_pagination(
                    address,
                    cursor.as_ref(),
                    request_inner.ledger_version.unwrap_or(ledger_version),
                    remaining_to_get as u64,
                )
                .expect("Failed to get resources from storage");
            let resources: HashMap<StructTag, Vec<u8>> = resources.into_iter().collect();
            remaining_to_get -= resources.len() as u16;
            raw_out.entry(address).or_insert(resources);
            cursor = next_state_key;
            // If there is no cursor, we can move on to the next address.
            if cursor.is_none() {
                address_index += 1;
            }
            if remaining_to_get == 0 {
                break;
            }
        }
        match cursor {
            Some(cursor) => Some(GetResourcesPageToken::AllResourceTypes((
                address_index,
                Some(cursor.into_inner()),
            ))),
            None => {
                if address_index == len as u32 {
                    None
                } else {
                    Some(GetResourcesPageToken::AllResourceTypes((
                        address_index,
                        None,
                    )))
                }
            },
        }
    } else {
        let mut resources = Vec::new();
        for resource in request_inner.resource_types.iter() {
            let resource = StructTag::from_str(resource)
                .map_err(|e| ApiError::new(Code::InvalidArgument, e.to_string()))?;
            resources.push(resource);
        }
        let (gets, cursor) =
            calculate_next_page_of_resource_gets(addresses, resources, limit, page_token)?;
        let mut tasks = Vec::new();
        for (address, resource) in gets {
            let state_view = state_view.clone();
            tasks.push(tokio::task::spawn_blocking(move || {
                resource_point_get(state_view, address, resource)
            }));
        }
        let mut results = Vec::new();
        for h in join_all(tasks).await {
            let result =
                h.map_err(|e| ApiError::new(Code::Internal, "Failed to join task".to_string()))?;
            results.push(result?);
        }
        for (address, resource, value) in results {
            raw_out
                .entry(address)
                .or_insert(HashMap::new())
                .insert(resource, value);
        }
        cursor
    };

    let mut out: HashMap<String, Resources> = HashMap::new();
    if request_inner.raw {
        for (address, resources) in raw_out {
            let inner = resources
                .into_iter()
                .map(|(resource, value)| {
                    (resource.to_canonical_string(), ResourceWrapper {
                        response: Some(resource_wrapper::Response::Raw(value)),
                    })
                })
                .collect();
            out.insert(address.to_canonical_string(), Resources {
                resources: inner,
            });
        }
    } else {
        let mut tasks: Vec<
            tokio::task::JoinHandle<Result<(AccountAddress, StructTag, MoveResource)>>,
        > = Vec::new();
        // TODO: I thought this was slow, but I think this actually does no I/O.
        for (address, resources) in raw_out {
            for (resource_type, value) in resources {
                let state_view = state_view.clone();
                let db = service.context.db.clone();
                tasks.push(tokio::task::spawn_blocking(move || {
                    let resource = state_view
                        .as_move_resolver()
                        .as_converter(db)
                        .try_into_resource(&resource_type, &value)?;
                    Ok((address, resource_type, resource))
                }));
            }
        }
        let mut results: Vec<(AccountAddress, StructTag, MoveResource)> = Vec::new();
        for h in join_all(tasks).await {
            let result =
                h.map_err(|_| ApiError::new(Code::Internal, "Failed to join task".to_string()))?;
            let result = result.map_err(|e| ApiError::new(Code::Internal, e.to_string()))?;
            results.push(result);
        }
        for (account_address, resource_type, move_resource) in results {
            // https://github.com/influxdata/pbjson/issues/96
            let data: HashMap<String, pbjson_types::Value> = move_resource
                .data
                .0
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        pbjson_types::Value::from(serde_json::to_string(&v).unwrap()),
                    )
                })
                .collect();
            let wrapper = ResourceWrapper {
                response: Some(resource_wrapper::Response::Parsed(
                    pbjson_types::Value::from(data),
                )),
            };
            out.entry(account_address.to_canonical_string())
                .or_insert_with(|| Resources {
                    resources: HashMap::new(),
                })
                .resources
                .insert(resource_type.to_canonical_string(), wrapper);
        }
    };

    let next_page_token = cursor.map(|c| bcs::to_bytes(&c).unwrap());
    let response = Response::new(GetResourcesResponse {
        resources: out,
        next_page_token,
    });
    println!("response: {:?}", response);
    Ok(response)
}

// Figure out which specific resources to get in this "page". We return the new page
// token if there is one.
fn calculate_next_page_of_resource_gets(
    accounts: Vec<AccountAddress>,
    resources: Vec<StructTag>,
    limit: u16,
    token: Option<GetResourcesPageToken>,
) -> Result<
    (
        Vec<(AccountAddress, StructTag)>,
        Option<GetResourcesPageToken>,
    ),
    ApiError,
> {
    let target_index = token
        .map(|t| match t {
            GetResourcesPageToken::AllResourceTypes(_) => Err(ApiError::new(
                Code::InvalidArgument,
                "Unexpected token type",
            )),
            GetResourcesPageToken::SpecificResourceTypes(index) => Ok(index),
        })
        .transpose()?
        .unwrap_or(0) as u16;
    let mut out = Vec::new();
    let mut count = 0;
    let accounts_len = accounts.len() as u16;
    let resources_len = resources.len() as u16;
    'outer: for account in accounts {
        for resource in resources.iter() {
            if count < target_index {
                count += 1;
                continue;
            }
            out.push((account, resource.clone()));
            count += 1;
            if count == target_index + limit as u16 {
                break 'outer;
            }
        }
    }
    println!(
        "{} {} {} {:?} {:?}",
        count,
        limit,
        out.len(),
        target_index,
        out
    );
    let token = if out.len() as u16 == limit && count < accounts_len * resources_len {
        Some(GetResourcesPageToken::SpecificResourceTypes(
            (target_index + limit) as u32,
        ))
    } else {
        None
    };
    Ok((out, token))
}

// Get a resource by address and name.
fn resource_point_get(
    state_view: Arc<DbStateView>,
    address: AccountAddress,
    resource_type: StructTag,
) -> Result<(AccountAddress, StructTag, Vec<u8>), ApiError> {
    let resolver = state_view.as_move_resolver();
    let bytes = resolver
        .get_resource(&address.into(), &resource_type)
        .context(format!(
            "Failed to query DB to check for {} at {}",
            resource_type, address
        ))
        .map_err(|e| ApiError::new(Code::Internal, e.to_string()))?
        .ok_or_else(|| ApiError::new(Code::NotFound, "Resource not found"))?;

    Ok((address, resource_type, bytes))
}
