// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use async_graphql::{Enum, InputValueError, Object, Scalar, ScalarType, SimpleObject, Union};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Module {
    module_abi: ModuleAbi,
}

#[Object]
impl Module {
    async fn module_abi(&self) -> &ModuleAbi {
        &self.module_abi
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct ModuleAbi {
    address: Address,
    name: String,
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub enum MoveLeafType {
    Unspecified,
    U16,
    U32,
    U64,
    U128,
    U256,
    Address,
    Signer,
}

#[derive(Clone, Debug, Union)]
pub enum MoveType {
    Vector(MoveTypeWrapper),
    GenericTypeParamIndex(GenericTypeParamIndex),
    ReferenceType(ReferenceType),
    StructTag(MoveStructTag),
    LeafType(MoveLeafTypeWrapper),
    FunctionType(MoveFunctionType),
}

/// This lets use `MoveType` in a union. This is necessary because we can't use
/// `Arc<MoveType>` directly in a union.
#[derive(Clone, Debug, SimpleObject)]
pub struct MoveTypeWrapper {
    pub value: Arc<MoveType>,
}

/// This is necessary because we can't used scalars directly in a union.
#[derive(Clone, Debug, SimpleObject)]
pub struct GenericTypeParamIndex {
    pub value: u32,
}

/// This is necessary because we can't use enums directly in a union.
#[derive(Clone, Debug, SimpleObject)]
pub struct MoveLeafTypeWrapper {
    value: MoveLeafType,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct ReferenceType {
    pub mutable: bool,
    pub to: Arc<MoveType>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct MoveStructTag {
    pub address: String,
    pub module: String,
    pub name: String,
    pub generic_type_params: Vec<Arc<MoveType>>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct MoveFunctionType {
    pub argument_types: Vec<Arc<MoveType>>,
    pub result_types: Vec<Arc<MoveType>>,
}

#[derive(Clone, Debug)]
pub struct Address(String);

#[Scalar]
impl ScalarType for Address {
    fn parse(value: async_graphql::Value) -> Result<Self, InputValueError<Address>> {
        match value {
            async_graphql::Value::String(s) => Ok(Self(s)),
            _ => Err("Invalid address format".into()),
        }
    }

    fn to_value(&self) -> async_graphql::Value {
        // Always return this as a canonical string. See the address standard.
        async_graphql::Value::String(self.0.clone())
    }
}

// Assuming MoveAbility and Visibility are Enums
#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub enum MoveAbility {
    Copy,
    Drop,
    Store,
    Key,
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Script,
    Friend,
    Private,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct Function {
    pub name: String,
    pub generic_type_params: Vec<FunctionGenericTypeParam>,
    pub args: Vec<FunctionArgument>,
    pub return_types: Vec<Arc<MoveType>>,
    pub visibility: Visibility,
    pub is_entry: bool,
    pub is_view: bool,
    pub comment: String,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct FunctionGenericTypeParam {
    pub name: String,
    pub constraints: Vec<MoveAbility>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct FunctionArgument {
    pub name: String,
    pub type_: Arc<MoveType>,
    pub comment: String,
}

// Assuming `Ability` is an Enum
#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub enum Ability {
    Copy,
    Drop,
    Store,
    Key,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct Struct {
    pub name: String,
    pub abilities: Vec<Ability>,
    pub generic_type_params: Vec<StructGenericTypeParam>,
    pub fields: Vec<StructField>,
    // TODO: Add resource group.
    pub comment: String,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct StructGenericTypeParam {
    pub name: String,
    pub constraints: Vec<MoveAbility>,
    pub is_phantom: bool,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct StructField {
    pub name: String,
    pub type_: Arc<MoveType>,
    pub comment: String,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct ErrorCode {
    pub name: String,
    pub code: u64,
    pub comment: String,
}
