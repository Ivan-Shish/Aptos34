// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::service::ApiV2Service;
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
    move_module_wrapper, GetAccountModulesRequest, GetAccountModulesResponse, MoveModuleWrapper,
    FILE_DESCRIPTOR_SET,
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
use std::{collections::HashMap, convert::TryInto, str::FromStr, sync::Arc};
use tonic::{transport::Server, Request, Response, Status};

// https://github.com/hyperium/tonic/discussions/1324

// advantages:
// better errors, no awkward mapping to status codes
// no U64 stuff, the types are real
// the cursor for pagination is returned as is
pub async fn get_account_modules(
    service: &ApiV2Service,
    request: Request<GetAccountModulesRequest>,
) -> Result<Response<GetAccountModulesResponse>, Status> {
    println!("Received request: {:?}", request);
    let inner = request.into_inner();
    let (ledger_info, ledger_version, state_view) = service
        .context
        .state_view::<BasicErrorWith404>(inner.ledger_version)
        .unwrap();
    let account_address = AccountAddress::from_hex_literal(&inner.account_address).unwrap();
    let (modules, next_state_key) = if inner.module_names.is_empty() {
        let max_account_modules_page_size = service.context.max_account_modules_page_size();
        let (modules, next_state_key) = service
            .context
            .get_modules_by_pagination(
                account_address,
                inner
                    .page_token
                    .map(|t| StateKey::new(bcs::from_bytes::<StateKeyInner>(&t).unwrap()))
                    .as_ref(),
                inner.ledger_version.unwrap_or(ledger_version),
                // Just use the max as the default
                determine_limit::<BasicErrorWith404>(
                    inner.page_size.map(|s| s as u16),
                    max_account_modules_page_size,
                    max_account_modules_page_size,
                    &ledger_info,
                )
                .unwrap() as u64,
            )
            .expect("Failed to get modules from storage");
        (
            modules
                .into_iter()
                .map(|m| (m.0.name().to_string(), m.1))
                .collect(),
            next_state_key,
        )
    } else {
        let mut tasks = Vec::new();
        let module_names = inner.module_names.iter().cloned();
        let state_view = Arc::new(state_view);
        for module_name in module_names {
            let state_view = state_view.clone();
            tasks.push(tokio::task::spawn_blocking(move || {
                module_point_get(state_view, account_address, module_name.clone())
            }));
        }
        let results: Vec<(String, Vec<u8>)> = join_all(tasks)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        (results, None)
    };

    let modules = if inner.raw {
        let mut out: HashMap<String, MoveModuleWrapper> = HashMap::new();
        for (module_name, module_bytes) in modules.into_iter() {
            let wrapper = MoveModuleWrapper {
                response: Some(move_module_wrapper::Response::Raw(module_bytes)),
            };
            out.insert(module_name, wrapper);
        }
        out
    } else {
        let mut tasks = Vec::new();
        // TODO: I thought this was slow, but I think this actually does no I/O.
        for (module_name, module_bytes) in modules.into_iter() {
            tasks.push(tokio::task::spawn_blocking(move || {
                let module = MoveModuleBytecode::new(module_bytes)
                    .try_parse_abi()
                    .expect("Failed to parse move module ABI from bytes retrieved from storage");
                (module_name, module)
            }));
        }
        let results: Vec<(String, MoveModuleBytecode)> = join_all(tasks)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        let mut out: HashMap<String, MoveModuleWrapper> = HashMap::new();
        for (module_name, module) in results {
            let module_proto = convert_move_module_bytecode(&module);
            let wrapper = MoveModuleWrapper {
                response: Some(move_module_wrapper::Response::Parsed(module_proto)),
            };
            out.insert(module_name, wrapper);
        }
        out
    };

    Ok(Response::new(GetAccountModulesResponse { modules }))
}

// Get a module by address and name.
fn module_point_get(
    state_view: Arc<DbStateView>,
    account_address: AccountAddress,
    module_name: String,
) -> (String, Vec<u8>) {
    let module_id = ModuleId::new(
        account_address,
        Identifier::new(module_name.clone()).unwrap(),
    );
    let access_path = AccessPath::code_access_path(module_id.clone());
    let state_key = StateKey::access_path(access_path);
    let bytes = state_view
        .get_state_value_bytes(&state_key)
        .expect(&format!("Failed to query DB to check for {:?}", state_key))
        .expect("Result was None");
    (module_name, bytes)
}
