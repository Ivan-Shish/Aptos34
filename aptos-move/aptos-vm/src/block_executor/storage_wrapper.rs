// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter_common::PreprocessedTransaction,
    data_cache::{IntoMoveResolver, StorageAdapterOwned},
};
use aptos_block_executor::task::DataView as BlockExecutorDataView;
use aptos_state_view::{StateView, StateViewId};
use aptos_types::state_store::{state_key::StateKey, state_storage_usage::StateStorageUsage};

/// Type used for accessing base (storage) values.
pub(crate) struct AptosDataView<'a, S>(pub(crate) &'a S);

impl<'a, S: StateView> BlockExecutorDataView for AptosDataView<'a, S> {
    type T = PreprocessedTransaction;

    /// Gets the state value for a given state key.
    fn get_state_value(&self, state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        self.0.get_state_value(state_key)
    }

    fn id(&self) -> StateViewId {
        self.0.id()
    }
}

/// Type used for accessing latest values (i.e. values written by the highest transaction in
/// the block, o.w. the base value from storage)
pub(crate) struct AptosVersionedView<'a, S: StateView, D: BlockExecutorDataView + ?Sized> {
    state_view: &'a S,
    data_view: &'a D,
}

impl<'a, S: StateView, D: BlockExecutorDataView<T = PreprocessedTransaction> + ?Sized>
    AptosVersionedView<'a, S, D>
{
    pub fn new_view(
        state_view: &'a S,
        data_view: &'a D,
    ) -> StorageAdapterOwned<AptosVersionedView<'a, S, D>> {
        AptosVersionedView {
            state_view,
            data_view,
        }
        .into_move_resolver()
    }
}

impl<'a, S: StateView, D: BlockExecutorDataView<T = PreprocessedTransaction> + ?Sized> StateView
    for AptosVersionedView<'a, S, D>
{
    fn id(&self) -> StateViewId {
        assert_eq!(
            self.state_view.id(),
            self.data_view.id(),
            "Data view used by block executor does not match the state view used for storage"
        );
        self.state_view.id()
    }

    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get_state_value(&self, state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        self.data_view.get_state_value(state_key)
    }

    fn is_genesis(&self) -> bool {
        self.state_view.is_genesis()
    }

    fn get_usage(&self) -> anyhow::Result<StateStorageUsage> {
        self.state_view.get_usage()
    }
}
