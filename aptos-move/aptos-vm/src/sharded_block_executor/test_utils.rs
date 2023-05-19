// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_crypto::{ed25519::ed25519_keys::Ed25519PrivateKey, PrivateKey, SigningKey, Uniform};
use aptos_types::{
    chain_id::ChainId,
    transaction::{
        analyzed_transaction::AnalyzedTransaction, EntryFunction, RawTransaction, Script,
        SignedTransaction, Transaction, TransactionPayload,
    },
    utility_coin::APTOS_COIN_TYPE,
};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};

#[derive(Clone, Debug)]
pub struct TestAccount {
    pub account_address: AccountAddress,
    pub private_key: Ed25519PrivateKey,
}

pub fn generate_test_account() -> TestAccount {
    TestAccount {
        account_address: AccountAddress::random(),
        private_key: Ed25519PrivateKey::generate_for_testing(),
    }
}

pub fn create_no_dependency_transaction(num_transactions: usize) -> Vec<AnalyzedTransaction> {
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let public_key = private_key.public_key();
    let sender = AccountAddress::random();

    let mut transactions = Vec::new();

    for i in 0..num_transactions {
        let transaction_payload = TransactionPayload::Script(Script::new(vec![], vec![], vec![]));
        let raw_transaction = RawTransaction::new(
            sender,
            i as u64,
            transaction_payload,
            0,
            0,
            0,
            ChainId::new(10),
        );
        let txn = Transaction::UserTransaction(SignedTransaction::new(
            raw_transaction.clone(),
            public_key.clone(),
            private_key.sign(&raw_transaction).unwrap(),
        ));
        transactions.push(txn.into())
    }
    transactions
}

pub fn create_non_conflicting_p2p_transaction() -> AnalyzedTransaction {
    // create unique sender and receiver accounts so that there is no conflict
    let sender = generate_test_account();
    let receiver = generate_test_account();
    create_signed_p2p_transaction(sender, vec![receiver]).remove(0)
}

pub fn create_signed_p2p_transaction(
    sender: TestAccount,
    receivers: Vec<TestAccount>,
) -> Vec<AnalyzedTransaction> {
    let mut transactions = Vec::new();
    for (i, receiver) in receivers.iter().enumerate() {
        let transaction_payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::ONE, Identifier::new("coin").unwrap()),
            Identifier::new("transfer").unwrap(),
            vec![APTOS_COIN_TYPE.clone()],
            vec![
                bcs::to_bytes(&receiver.account_address).unwrap(),
                bcs::to_bytes(&1u64).unwrap(),
            ],
        ));

        let raw_transaction = RawTransaction::new(
            sender.account_address,
            i as u64,
            transaction_payload,
            0,
            0,
            0,
            ChainId::new(10),
        );
        let txn = Transaction::UserTransaction(SignedTransaction::new(
            raw_transaction.clone(),
            sender.private_key.public_key().clone(),
            sender.private_key.sign(&raw_transaction).unwrap(),
        ));
        transactions.push(txn.into())
    }
    transactions
}
