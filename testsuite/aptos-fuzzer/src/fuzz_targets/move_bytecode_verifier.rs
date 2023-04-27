// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use aptos_proptest_helpers::ValueGenerator;
use move_binary_format::file_format::{
    empty_module, AbilitySet, Bytecode, CodeUnit, CompiledModule, Constant, FieldDefinition,
    FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex, ModuleHandleIndex,
    Signature, SignatureIndex, SignatureToken,
    SignatureToken::{Address, Bool, U128, U64},
    StructDefinition, StructFieldInformation, StructHandle, StructHandleIndex, TypeSignature,
    Visibility,
};
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use proptest::arbitrary::any;
use proptest_derive::Arbitrary;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct MoveBytecodeVerifierCodeUnit;

impl FuzzTargetImpl for MoveBytecodeVerifierCodeUnit {
    fn description(&self) -> &'static str {
        "Move Bytecode Verifier - CodeUnit"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(any::<CodeUnit>()))
    }

    fn fuzz(&self, data: &[u8]) {
        let code_unit = fuzz_data_to_value(data, any::<CodeUnit>());
        let mut module = empty_module();
        module.version = 5;

        module.struct_handles.push(StructHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            abilities: AbilitySet::ALL,
            type_parameters: vec![],
        });

        let fun_handle = FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(2),
            parameters: SignatureIndex(0),
            return_: SignatureIndex(1),
            type_parameters: vec![],
        };

        module.function_handles.push(fun_handle);

        module.signatures.pop();
        module.signatures.push(Signature(vec![
            Address, U64, Address, Address, U128, Address, U64, U64, U64,
        ]));
        module.signatures.push(Signature(vec![]));
        module
            .signatures
            .push(Signature(vec![Address, Bool, Address]));

        module.identifiers.extend(
            vec![
                Identifier::from_str("zf_hello_world").unwrap(),
                Identifier::from_str("awldFnU18mlDKQfh6qNfBGx8X").unwrap(),
                Identifier::from_str("aQPwJNHyAHpvJ").unwrap(),
                Identifier::from_str("aT7ZphKTrKcYCwCebJySrmrKlckmnL5").unwrap(),
                Identifier::from_str("arYpsFa2fvrpPJ").unwrap(),
            ]
            .into_iter(),
        );
        module.address_identifiers.push(AccountAddress::random());

        module.constant_pool.push(Constant {
            type_: Address,
            data: AccountAddress::ZERO.into_bytes().to_vec(),
        });

        module.struct_defs.push(StructDefinition {
            struct_handle: StructHandleIndex(0),
            field_information: StructFieldInformation::Declared(vec![FieldDefinition {
                name: IdentifierIndex::new(3),
                signature: TypeSignature(Address),
            }]),
        });

        let fun_def = FunctionDefinition {
            code: Some(code_unit),
            function: FunctionHandleIndex(0),
            visibility: Visibility::Public,
            is_entry: false,
            acquires_global_resources: vec![],
        };

        module.function_defs.push(fun_def);
        let _ = move_bytecode_verifier::verify_module(&module);
    }
}

#[derive(Debug, Default)]
pub struct MoveBytecodeVerifierCompiledModule;

impl FuzzTargetImpl for MoveBytecodeVerifierCompiledModule {
    fn description(&self) -> &'static str {
        "Move Bytecode Verifier - Compiled Module"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(any::<CompiledModule>()))
    }

    fn fuzz(&self, data: &[u8]) {
        let module = fuzz_data_to_value(data, any::<CompiledModule>());
        let _ = move_bytecode_verifier::verify_module(&module);
    }
}

#[derive(Debug, Default)]
pub struct MoveBytecodeVerifierMixed;

/// Mixed fuzz target for the Move bytecode verifier.
#[derive(Arbitrary, Debug)]
struct Mixed {
    code: Vec<Bytecode>,
    abilities: AbilitySet,
    param_types: Vec<SignatureToken>,
    return_type: Option<SignatureToken>,
}

impl FuzzTargetImpl for MoveBytecodeVerifierMixed {
    fn description(&self) -> &'static str {
        "Move Bytecode Verifier - Mixed"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(any::<Mixed>()))
    }

    fn fuzz(&self, data: &[u8]) {
        let mix = fuzz_data_to_value(data, any::<Mixed>());
        let mut module = empty_module();
        module.version = 5;

        module.struct_handles.push(StructHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            abilities: mix.abilities,
            type_parameters: vec![],
        });

        let fun_handle = FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(2),
            parameters: SignatureIndex(0),
            return_: SignatureIndex(1),
            type_parameters: vec![],
        };

        module.function_handles.push(fun_handle);

        module.signatures.pop();
        module.signatures.push(Signature(mix.param_types));
        module.signatures.push(Signature(
            mix.return_type.map(|s| vec![s]).unwrap_or_default(),
        ));
        module
            .signatures
            .push(Signature(vec![Address, Bool, Address]));

        module.identifiers.extend(
            vec![
                Identifier::from_str("zf_hello_world").unwrap(),
                Identifier::from_str("awldFnU18mlDKQfh6qNfBGx8X").unwrap(),
                Identifier::from_str("aQPwJNHyAHpvJ").unwrap(),
                Identifier::from_str("aT7ZphKTrKcYCwCebJySrmrKlckmnL5").unwrap(),
                Identifier::from_str("arYpsFa2fvrpPJ").unwrap(),
            ]
            .into_iter(),
        );
        module.address_identifiers.push(AccountAddress::random());

        module.constant_pool.push(Constant {
            type_: Address,
            data: AccountAddress::ZERO.into_bytes().to_vec(),
        });

        module.struct_defs.push(StructDefinition {
            struct_handle: StructHandleIndex(0),
            field_information: StructFieldInformation::Declared(vec![FieldDefinition {
                name: IdentifierIndex::new(3),
                signature: TypeSignature(Address),
            }]),
        });

        let code_unit = CodeUnit {
            code: mix.code,
            locals: SignatureIndex(0),
        };

        let fun_def = FunctionDefinition {
            code: Some(code_unit),
            function: FunctionHandleIndex(0),
            visibility: Visibility::Public,
            is_entry: false,
            acquires_global_resources: vec![],
        };

        module.function_defs.push(fun_def);
        let _ = move_bytecode_verifier::verify_module(&module);
    }
}
