// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use aptos_crypto::HashValue;
use aptos_storage_interface::{DbReader, ExecutedTrees, Order};
use aptos_types::{
    account_config::NewBlockEvent,
    contract_event::{ContractEvent, EventWithVersion},
    epoch_change::EpochChangeProof,
    epoch_state::EpochState,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    proof::{
        AccumulatorConsistencyProof, SparseMerkleProof, SparseMerkleProofExt,
        TransactionAccumulatorRangeProof, TransactionAccumulatorSummary,
    },
    state_proof::StateProof,
    state_store::{
        state_key::StateKey,
        state_key_prefix::StateKeyPrefix,
        state_storage_usage::StateStorageUsage,
        state_value::{StateValue, StateValueChunkWithProof},
        table::{TableHandle, TableInfo},
    },
    transaction::{
        AccountTransactionsWithProof, Transaction, TransactionInfo, TransactionListWithProof,
        TransactionOutputListWithProof, TransactionWithProof, Version,
    },
    write_set::WriteSet,
};
use arc_swap::ArcSwap;
use move_core_types::account_address::AccountAddress;
use std::sync::Arc;

// A simple type entry to represent each cache entry
type CacheEntry<T> = Arc<ArcSwap<Option<T>>>;

// A simple macro procedure that performs a cache lookup
// or insertion depending on the state of the cache entry.
macro_rules! perform_cache_lookup_or_insert {
    ($self:ident, $cache_entry_name:ident, $($args:tt),*) => {
        // Lookup the cache entry and return the value (if one exists).
        // Otherwise, fetch the value from storage, update the
        // cache entry and return the result.
        if let Some(result) = $self.$cache_entry_name.load().as_ref() {
            return Ok(result.clone());
        } else {
            let result = $self.get_db().$cache_entry_name($($args),*);
            if let Ok(result) = &result {
                ($self.$cache_entry_name).swap(Arc::new(Some(result.clone())));
            }
            return result
        }
    }
}

#[derive(Clone, Default)]
pub struct DatabaseWithCache {
    db: Option<Arc<dyn DbReader>>,

    // Cached entries
    get_block_timestamp: CacheEntry<u64>,
    get_block_info_by_height: CacheEntry<(Version, Version, NewBlockEvent)>,
    get_block_info_by_version: CacheEntry<(Version, Version, NewBlockEvent)>,
    get_epoch_ending_ledger_infos: CacheEntry<EpochChangeProof>,
    get_events: CacheEntry<Vec<EventWithVersion>>,
    get_gas_prices: CacheEntry<Vec<u64>>,
    get_first_txn_version: CacheEntry<Option<Version>>,
    get_first_viable_txn_version: CacheEntry<Version>,
    get_first_write_set_version: CacheEntry<Option<Version>>,
    get_last_version_before_timestamp: CacheEntry<Version>,
    get_latest_epoch_state: CacheEntry<EpochState>,
    get_latest_ledger_info: CacheEntry<LedgerInfoWithSignatures>,
    get_latest_ledger_info_option: CacheEntry<Option<LedgerInfoWithSignatures>>,
    get_latest_version: CacheEntry<Version>,
    get_next_block_event: CacheEntry<(Version, NewBlockEvent)>,
    get_transaction_accumulator_range_proof: CacheEntry<TransactionAccumulatorRangeProof>,
    get_transactions: CacheEntry<TransactionListWithProof>,
    get_transaction_by_hash: CacheEntry<Option<TransactionWithProof>>,
    get_transaction_by_version: CacheEntry<TransactionWithProof>,
    get_transaction_outputs: CacheEntry<TransactionOutputListWithProof>,
}

impl DatabaseWithCache {
    pub fn new(db: Arc<dyn DbReader>) -> Self {
        Self {
            db: Some(db),
            ..Default::default()
        }
    }

    /// Utility function that returns the database
    fn get_db(&self) -> &Arc<dyn DbReader> {
        self.db.as_ref().expect("The db must be initialized!")
    }
}

impl DbReader for DatabaseWithCache {
    fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<EpochChangeProof> {
        perform_cache_lookup_or_insert!(self, get_epoch_ending_ledger_infos, start_epoch, end_epoch)
    }

    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        perform_cache_lookup_or_insert!(
            self,
            get_transactions,
            start_version,
            batch_size,
            ledger_version,
            fetch_events
        )
    }

    fn get_gas_prices(
        &self,
        start_version: Version,
        limit: u64,
        ledger_version: Version,
    ) -> Result<Vec<u64>> {
        perform_cache_lookup_or_insert!(self, get_gas_prices, start_version, limit, ledger_version)
    }

    fn get_transaction_by_hash(
        &self,
        hash: HashValue,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>> {
        perform_cache_lookup_or_insert!(
            self,
            get_transaction_by_hash,
            hash,
            ledger_version,
            fetch_events
        )
    }

    fn get_transaction_by_version(
        &self,
        version: Version,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionWithProof> {
        perform_cache_lookup_or_insert!(
            self,
            get_transaction_by_version,
            version,
            ledger_version,
            fetch_events
        )
    }

    fn get_first_txn_version(&self) -> Result<Option<Version>> {
        perform_cache_lookup_or_insert!(self, get_first_txn_version,)
    }

    fn get_first_viable_txn_version(&self) -> Result<Version> {
        perform_cache_lookup_or_insert!(self, get_first_viable_txn_version,)
    }

    fn get_first_write_set_version(&self) -> Result<Option<Version>> {
        perform_cache_lookup_or_insert!(self, get_first_write_set_version,)
    }

    fn get_transaction_outputs(
        &self,
        start_version: Version,
        limit: u64,
        ledger_version: Version,
    ) -> Result<TransactionOutputListWithProof> {
        perform_cache_lookup_or_insert!(
            self,
            get_transaction_outputs,
            start_version,
            limit,
            ledger_version
        )
    }

    fn get_events(
        &self,
        event_key: &EventKey,
        start: u64,
        order: Order,
        limit: u64,
        ledger_version: Version,
    ) -> Result<Vec<EventWithVersion>> {
        perform_cache_lookup_or_insert!(
            self,
            get_events,
            event_key,
            start,
            order,
            limit,
            ledger_version
        )
    }

    fn get_transaction_iterator(
        &self,
        start_version: Version,
        limit: u64,
    ) -> Result<Box<dyn Iterator<Item = Result<Transaction>> + '_>> {
        self.get_db().get_transaction_iterator(start_version, limit)
    }

    fn get_transaction_info_iterator(
        &self,
        start_version: Version,
        limit: u64,
    ) -> Result<Box<dyn Iterator<Item = Result<TransactionInfo>> + '_>> {
        self.get_db()
            .get_transaction_info_iterator(start_version, limit)
    }

    fn get_events_iterator(
        &self,
        start_version: Version,
        limit: u64,
    ) -> Result<Box<dyn Iterator<Item = Result<Vec<ContractEvent>>> + '_>> {
        self.get_db().get_events_iterator(start_version, limit)
    }

    fn get_write_set_iterator(
        &self,
        start_version: Version,
        limit: u64,
    ) -> Result<Box<dyn Iterator<Item = Result<WriteSet>> + '_>> {
        self.get_db().get_write_set_iterator(start_version, limit)
    }

    fn get_transaction_accumulator_range_proof(
        &self,
        start_version: Version,
        limit: u64,
        ledger_version: Version,
    ) -> Result<TransactionAccumulatorRangeProof> {
        perform_cache_lookup_or_insert!(
            self,
            get_transaction_accumulator_range_proof,
            start_version,
            limit,
            ledger_version
        )
    }

    fn get_block_timestamp(&self, version: Version) -> Result<u64> {
        perform_cache_lookup_or_insert!(self, get_block_timestamp, version)
    }

    fn get_next_block_event(&self, version: Version) -> Result<(Version, NewBlockEvent)> {
        perform_cache_lookup_or_insert!(self, get_next_block_event, version)
    }

    fn get_block_info_by_version(
        &self,
        version: Version,
    ) -> Result<(Version, Version, NewBlockEvent)> {
        perform_cache_lookup_or_insert!(self, get_block_info_by_version, version)
    }

    fn get_block_info_by_height(&self, height: u64) -> Result<(Version, Version, NewBlockEvent)> {
        perform_cache_lookup_or_insert!(self, get_block_info_by_height, height)
    }

    fn get_last_version_before_timestamp(
        &self,
        timestamp: u64,
        ledger_version: Version,
    ) -> Result<Version> {
        perform_cache_lookup_or_insert!(
            self,
            get_last_version_before_timestamp,
            timestamp,
            ledger_version
        )
    }

    fn get_latest_epoch_state(&self) -> Result<EpochState> {
        perform_cache_lookup_or_insert!(self, get_latest_epoch_state,)
    }

    fn get_prefixed_state_value_iterator(
        &self,
        key_prefix: &StateKeyPrefix,
        cursor: Option<&StateKey>,
        version: Version,
    ) -> Result<Box<dyn Iterator<Item = Result<(StateKey, StateValue)>> + '_>> {
        self.get_db()
            .get_prefixed_state_value_iterator(key_prefix, cursor, version)
    }

    fn get_latest_ledger_info_option(&self) -> Result<Option<LedgerInfoWithSignatures>> {
        perform_cache_lookup_or_insert!(self, get_latest_ledger_info_option,)
    }

    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        perform_cache_lookup_or_insert!(self, get_latest_ledger_info,)
    }

    fn get_latest_version(&self) -> Result<Version> {
        perform_cache_lookup_or_insert!(self, get_latest_version,)
    }

    fn get_latest_state_checkpoint_version(&self) -> Result<Option<Version>> {
        self.get_db().get_latest_state_checkpoint_version()
    }

    fn get_state_snapshot_before(
        &self,
        next_version: Version,
    ) -> Result<Option<(Version, HashValue)>> {
        self.get_db().get_state_snapshot_before(next_version)
    }

    fn get_latest_commit_metadata(&self) -> Result<(Version, u64)> {
        self.get_db().get_latest_commit_metadata()
    }

    fn get_account_transaction(
        &self,
        address: AccountAddress,
        seq_num: u64,
        include_events: bool,
        ledger_version: Version,
    ) -> Result<Option<TransactionWithProof>> {
        self.get_db()
            .get_account_transaction(address, seq_num, include_events, ledger_version)
    }

    fn get_account_transactions(
        &self,
        address: AccountAddress,
        seq_num: u64,
        limit: u64,
        include_events: bool,
        ledger_version: Version,
    ) -> Result<AccountTransactionsWithProof> {
        self.get_db().get_account_transactions(
            address,
            seq_num,
            limit,
            include_events,
            ledger_version,
        )
    }

    fn get_state_proof_with_ledger_info(
        &self,
        known_version: u64,
        ledger_info: LedgerInfoWithSignatures,
    ) -> Result<StateProof> {
        self.get_db()
            .get_state_proof_with_ledger_info(known_version, ledger_info)
    }

    fn get_state_proof(&self, known_version: u64) -> Result<StateProof> {
        self.get_db().get_state_proof(known_version)
    }

    fn get_state_value_by_version(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<Option<StateValue>> {
        self.get_db().get_state_value_by_version(state_key, version)
    }

    fn get_state_proof_by_version_ext(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<SparseMerkleProofExt> {
        self.get_db()
            .get_state_proof_by_version_ext(state_key, version)
    }

    fn get_state_value_with_proof_by_version_ext(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<(Option<StateValue>, SparseMerkleProofExt)> {
        self.get_db()
            .get_state_value_with_proof_by_version_ext(state_key, version)
    }

    fn get_state_value_with_proof_by_version(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<(Option<StateValue>, SparseMerkleProof)> {
        self.get_db()
            .get_state_value_with_proof_by_version(state_key, version)
    }

    fn get_latest_executed_trees(&self) -> Result<ExecutedTrees> {
        self.get_db().get_latest_executed_trees()
    }

    fn get_epoch_ending_ledger_info(&self, known_version: u64) -> Result<LedgerInfoWithSignatures> {
        self.get_db().get_epoch_ending_ledger_info(known_version)
    }

    fn get_latest_transaction_info_option(&self) -> Result<Option<(Version, TransactionInfo)>> {
        self.get_db().get_latest_transaction_info_option()
    }

    fn get_accumulator_root_hash(&self, version: Version) -> Result<HashValue> {
        self.get_db().get_accumulator_root_hash(version)
    }

    fn get_accumulator_consistency_proof(
        &self,
        client_known_version: Option<Version>,
        ledger_version: Version,
    ) -> Result<AccumulatorConsistencyProof> {
        self.get_db()
            .get_accumulator_consistency_proof(client_known_version, ledger_version)
    }

    fn get_accumulator_summary(
        &self,
        ledger_version: Version,
    ) -> Result<TransactionAccumulatorSummary> {
        self.get_db().get_accumulator_summary(ledger_version)
    }

    fn get_state_leaf_count(&self, version: Version) -> Result<usize> {
        self.get_db().get_state_leaf_count(version)
    }

    fn get_state_value_chunk_with_proof(
        &self,
        version: Version,
        start_idx: usize,
        chunk_size: usize,
    ) -> Result<StateValueChunkWithProof> {
        self.get_db()
            .get_state_value_chunk_with_proof(version, start_idx, chunk_size)
    }

    fn is_state_merkle_pruner_enabled(&self) -> Result<bool> {
        self.get_db().is_state_merkle_pruner_enabled()
    }

    fn get_epoch_snapshot_prune_window(&self) -> Result<usize> {
        self.get_db().get_epoch_snapshot_prune_window()
    }

    fn is_ledger_pruner_enabled(&self) -> Result<bool> {
        self.get_db().is_ledger_pruner_enabled()
    }

    fn get_ledger_prune_window(&self) -> Result<usize> {
        self.get_db().get_ledger_prune_window()
    }

    fn get_table_info(&self, handle: TableHandle) -> Result<TableInfo> {
        self.get_db().get_table_info(handle)
    }

    fn indexer_enabled(&self) -> bool {
        self.get_db().indexer_enabled()
    }

    fn get_state_storage_usage(&self, version: Option<Version>) -> Result<StateStorageUsage> {
        self.get_db().get_state_storage_usage(version)
    }
}
