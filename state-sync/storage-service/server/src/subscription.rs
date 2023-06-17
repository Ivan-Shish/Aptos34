// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    metrics,
    metrics::{increment_counter, SUBSCRIPTION_EXPIRE},
    moderator::RequestModerator,
    network::ResponseSender,
    optimistic_fetch::OptimisticFetchRequest,
    storage::StorageReaderInterface,
    utils, LogEntry, LogSchema,
};
use aptos_config::{config::StorageServiceConfig, network_id::PeerNetworkId};
use aptos_infallible::{Mutex, RwLock};
use aptos_logger::warn;
use aptos_storage_service_types::{
    requests::{
        DataRequest, StorageServiceRequest, TransactionOutputsWithProofRequest,
        TransactionsOrOutputsWithProofRequest, TransactionsWithProofRequest,
    },
    responses::{DataResponse, StorageServerSummary, StorageServiceResponse},
};
use aptos_time_service::{TimeService, TimeServiceTrait};
use aptos_types::ledger_info::LedgerInfoWithSignatures;
use lru::LruCache;
use std::{
    cmp::min,
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Instant,
};

/// A single subscription request that is part of a stream
pub struct SubscriptionRequest {
    request: StorageServiceRequest,  // The original request
    response_sender: ResponseSender, // The sender along which to send the response
    request_start_time: Instant,     // The time the request started (i.e., when it was received)
}

impl SubscriptionRequest {
    pub fn new(
        request: StorageServiceRequest,
        response_sender: ResponseSender,
        time_service: TimeService,
    ) -> Self {
        Self {
            request,
            response_sender,
            request_start_time: time_service.now(),
        }
    }

    /// Creates a new storage service request to satisfy the request
    /// using the new data at the specified `target_ledger_info`.
    fn get_storage_request_for_missing_data(
        &self,
        config: StorageServiceConfig,
        known_version: u64,
        target_ledger_info: &LedgerInfoWithSignatures,
    ) -> aptos_storage_service_types::Result<StorageServiceRequest, Error> {
        // Calculate the number of versions to fetch
        let target_version = target_ledger_info.ledger_info().version();
        let mut num_versions_to_fetch =
            target_version.checked_sub(known_version).ok_or_else(|| {
                Error::UnexpectedErrorEncountered(
                    "Number of versions to fetch has overflown!".into(),
                )
            })?;

        // Bound the number of versions to fetch by the maximum chunk size
        num_versions_to_fetch = min(
            num_versions_to_fetch,
            self.max_chunk_size_for_request(config),
        );

        // Calculate the start and end versions
        let start_version = known_version.checked_add(1).ok_or_else(|| {
            Error::UnexpectedErrorEncountered("Start version has overflown!".into())
        })?;
        let end_version = known_version
            .checked_add(num_versions_to_fetch)
            .ok_or_else(|| {
                Error::UnexpectedErrorEncountered("End version has overflown!".into())
            })?;

        // Create the storage request
        let data_request = match &self.request.data_request {
            DataRequest::SubscribeTransactionOutputsWithProof(_) => {
                DataRequest::GetTransactionOutputsWithProof(TransactionOutputsWithProofRequest {
                    proof_version: target_version,
                    start_version,
                    end_version,
                })
            },
            DataRequest::SubscribeTransactionsWithProof(request) => {
                DataRequest::GetTransactionsWithProof(TransactionsWithProofRequest {
                    proof_version: target_version,
                    start_version,
                    end_version,
                    include_events: request.include_events,
                })
            },
            DataRequest::SubscribeTransactionsOrOutputsWithProof(request) => {
                DataRequest::GetTransactionsOrOutputsWithProof(
                    TransactionsOrOutputsWithProofRequest {
                        proof_version: target_version,
                        start_version,
                        end_version,
                        include_events: request.include_events,
                        max_num_output_reductions: request.max_num_output_reductions,
                    },
                )
            },
            request => unreachable!("Unexpected subscription request: {:?}", request),
        };
        let storage_request =
            StorageServiceRequest::new(data_request, self.request.use_compression);
        Ok(storage_request)
    }

    /// Returns the highest version known by the peer
    fn highest_known_version(&self) -> u64 {
        match &self.request.data_request {
            DataRequest::SubscribeTransactionOutputsWithProof(request) => request.known_version,
            DataRequest::SubscribeTransactionsWithProof(request) => request.known_version,
            DataRequest::SubscribeTransactionsOrOutputsWithProof(request) => request.known_version,
            request => unreachable!("Unexpected subscription request: {:?}", request),
        }
    }

    /// Returns the highest epoch known by the peer
    fn highest_known_epoch(&self) -> u64 {
        match &self.request.data_request {
            DataRequest::SubscribeTransactionOutputsWithProof(request) => request.known_epoch,
            DataRequest::SubscribeTransactionsWithProof(request) => request.known_epoch,
            DataRequest::SubscribeTransactionsOrOutputsWithProof(request) => request.known_epoch,
            request => unreachable!("Unexpected subscription request: {:?}", request),
        }
    }

    /// Returns the maximum chunk size for the request
    /// depending on the request type.
    fn max_chunk_size_for_request(&self, config: StorageServiceConfig) -> u64 {
        match &self.request.data_request {
            DataRequest::SubscribeTransactionOutputsWithProof(_) => {
                config.max_transaction_output_chunk_size
            },
            DataRequest::SubscribeTransactionsWithProof(_) => config.max_transaction_chunk_size,
            DataRequest::SubscribeTransactionsOrOutputsWithProof(_) => {
                config.max_transaction_output_chunk_size
            },
            request => unreachable!("Unexpected subscription request: {:?}", request),
        }
    }

    /// Returns the subscription stream id for the request
    pub fn subscription_stream_id(&self) -> u64 {
        match &self.request.data_request {
            DataRequest::SubscribeTransactionOutputsWithProof(request) => {
                request.subscription_stream_index
            },
            DataRequest::SubscribeTransactionsWithProof(request) => {
                request.subscription_stream_index
            },
            DataRequest::SubscribeTransactionsOrOutputsWithProof(request) => {
                request.subscription_stream_index
            },
            request => unreachable!("Unexpected subscription request: {:?}", request),
        }
    }

    /// Returns the subscription stream index for the request
    fn subscription_stream_index(&self) -> u64 {
        match &self.request.data_request {
            DataRequest::SubscribeTransactionOutputsWithProof(request) => {
                request.subscription_stream_id
            },
            DataRequest::SubscribeTransactionsWithProof(request) => request.subscription_stream_id,
            DataRequest::SubscribeTransactionsOrOutputsWithProof(request) => {
                request.subscription_stream_id
            },
            request => unreachable!("Unexpected subscription request: {:?}", request),
        }
    }
}

/// A set of subscription requests that together form a stream
pub struct SubscriptionStreamRequests {
    highest_known_version: u64, // The highest version known by the peer (at this point in the stream)
    highest_known_epoch: u64,   // The highest epoch known by the peer (at this point in the stream)

    next_index_to_serve: u64, // The next subscription stream request index to serve
    pending_subscription_requests: BTreeMap<u64, SubscriptionRequest>, // The pending subscription requests by stream index
    subscription_stream_id: u64, // The unique stream ID (as specified by the client)

    last_stream_update_time: Instant, // The last time the stream was updated
    time_service: TimeService,        // The time service
}

impl SubscriptionStreamRequests {
    pub fn new(subscription_request: SubscriptionRequest, time_service: TimeService) -> Self {
        // Extract the relevant information from the request
        let highest_known_version = subscription_request.highest_known_version();
        let highest_known_epoch = subscription_request.highest_known_epoch();
        let subscription_stream_id = subscription_request.subscription_stream_id();

        // Create a new set of pending subscription requests using the first request
        let mut pending_subscription_requests = BTreeMap::new();
        pending_subscription_requests.insert(
            subscription_request.subscription_stream_index(),
            subscription_request,
        );

        Self {
            highest_known_version,
            highest_known_epoch,
            next_index_to_serve: 0,
            pending_subscription_requests,
            subscription_stream_id,
            last_stream_update_time: time_service.now(),
            time_service,
        }
    }

    /// Adds a subscription request to the existing stream.
    ///
    /// Note: This function assumes the caller has already verified that
    /// the stream ID's match for the request.
    pub fn add_subscription_request(
        &mut self,
        subscription_request: SubscriptionRequest,
    ) -> Result<(), Error> {
        // Verify that the subscription request index is valid
        let subscription_request_index = subscription_request.subscription_stream_index();
        if subscription_request_index < self.next_index_to_serve {
            return Err(Error::InvalidRequest(format!(
                "The subscription request index is too low! Next index to serve: {:?}, found: {:?}",
                self.next_index_to_serve, subscription_request_index
            )));
        }

        // Insert the subscription request into the pending requests
        let existing_request = self.pending_subscription_requests.insert(
            subscription_request.subscription_stream_index(),
            subscription_request,
        );

        // Refresh the last stream update time
        self.refresh_last_stream_update_time();

        // If a pending request already existed, return an error
        if existing_request.is_some() {
            return Err(Error::InvalidRequest(format!(
                "An existing subscription request was found for the given index: {:?}",
                subscription_request_index
            )));
        }

        Ok(())
    }

    /// Returns a reference to the first pending subscription request
    /// in the stream (if it exists).
    fn first_pending_request(&self) -> Option<&SubscriptionRequest> {
        self.pending_subscription_requests
            .first_key_value()
            .map(|(_, request)| request)
    }

    /// Returns true iff the subscription stream has expired.
    /// There are two ways a stream can expire: (i) the first
    /// pending request has been blocked for too long; or (ii)
    /// the stream has been idle for too long.
    fn is_expired(&self, timeout_ms: u64) -> bool {
        if let Some(subscription_request) = self.first_pending_request() {
            // Verify the stream hasn't been blocked for too long
            let current_time = self.time_service.now();
            let elapsed_time = current_time
                .duration_since(subscription_request.request_start_time)
                .as_millis();
            if elapsed_time > timeout_ms as u128 {
                return true; // The stream has been blocked for too long
            }
        } else {
            // If the steam is empty, verify the stream hasn't been idle for too long
            let current_time = self.time_service.now();
            let elapsed_time = current_time
                .duration_since(self.last_stream_update_time)
                .as_millis();
            if elapsed_time > timeout_ms as u128 {
                return true; // The stream has been idle for too long
            }
        }

        false
    }

    /// Removes the first pending subscription request from the stream
    /// and returns it (if it exists).
    fn pop_first_pending_request(&mut self) -> Option<SubscriptionRequest> {
        self.pending_subscription_requests
            .pop_first()
            .map(|(_, request)| request)
    }

    /// Refreshes the last stream update time to the current time
    fn refresh_last_stream_update_time(&mut self) {
        self.last_stream_update_time = self.time_service.now();
    }

    /// Returns the unique stream id for the stream
    pub fn subscription_stream_id(&self) -> u64 {
        self.subscription_stream_id
    }

    /// Updates the highest known version and epoch for the stream
    /// using the latest data response that was sent to the client.
    fn update_known_version_and_epoch(
        &mut self,
        data_response: &DataResponse,
    ) -> Result<(), Error> {
        // Determine the number of data items and target ledger info sent to the client
        let (num_data_items, target_ledger_info) = match data_response {
            DataResponse::NewTransactionOutputsWithProof((
                transaction_output_list,
                target_ledger_info,
            )) => (
                transaction_output_list.transactions_and_outputs.len(),
                target_ledger_info,
            ),
            DataResponse::NewTransactionsWithProof((transaction_list, target_ledger_info)) => {
                (transaction_list.transactions.len(), target_ledger_info)
            },
            DataResponse::NewTransactionsOrOutputsWithProof((
                (transaction_list, transaction_output_list),
                target_ledger_info,
            )) => {
                if let Some(transaction_list) = transaction_list {
                    (transaction_list.transactions.len(), target_ledger_info)
                } else if let Some(transaction_output_list) = transaction_output_list {
                    (
                        transaction_output_list.transactions_and_outputs.len(),
                        target_ledger_info,
                    )
                } else {
                    return Err(Error::UnexpectedErrorEncountered(format!(
                        "New transactions or outputs response is missing data: {:?}",
                        data_response
                    )));
                }
            },
            _ => {
                return Err(Error::UnexpectedErrorEncountered(format!(
                    "Unexpected data response type: {:?}",
                    data_response
                )))
            },
        };

        // Update the highest known version
        self.highest_known_version += num_data_items as u64;

        // Update the highest known epoch (iff an epoch change occurred)
        if target_ledger_info.ledger_info().ends_epoch() {
            self.highest_known_epoch += 1;
        }

        // Refresh the last stream update time
        self.refresh_last_stream_update_time();

        Ok(())
    }
}

/// Handles ready (and expired) subscriptions
pub(crate) fn handle_active_subscriptions<T: StorageReaderInterface>(
    cached_storage_server_summary: Arc<RwLock<StorageServerSummary>>,
    config: StorageServiceConfig,
    lru_response_cache: Arc<Mutex<LruCache<StorageServiceRequest, StorageServiceResponse>>>,
    optimistic_fetches: Arc<Mutex<HashMap<PeerNetworkId, OptimisticFetchRequest>>>,
    request_moderator: Arc<RequestModerator>,
    storage: T,
    subscriptions: Arc<Mutex<HashMap<PeerNetworkId, SubscriptionStreamRequests>>>,
    time_service: TimeService,
) -> Result<(), Error> {
    // Remove all expired subscription streams
    remove_expired_subscription_streams(config, subscriptions.clone());

    // Identify the peers with ready subscriptions
    let peers_with_ready_subscriptions = get_peers_with_ready_subscriptions(
        cached_storage_server_summary.clone(),
        optimistic_fetches.clone(),
        subscriptions.clone(),
        lru_response_cache.clone(),
        request_moderator.clone(),
        storage.clone(),
        time_service.clone(),
    )?;

    // Remove and handle the ready subscriptions
    for (peer, target_ledger_info) in peers_with_ready_subscriptions {
        if let Some(subscription_stream_requests) = subscriptions.lock().get_mut(&peer) {
            // The first request in the stream should be ready
            if let Some(subscription_request) =
                subscription_stream_requests.pop_first_pending_request()
            {
                let subscription_start_time = subscription_request.request_start_time;
                let subscription_data_request = subscription_request.request.clone();

                // Get the storage service request for the missing data and notify the peer
                let known_version = subscription_stream_requests.highest_known_version;
                match subscription_request.get_storage_request_for_missing_data(
                    config,
                    known_version,
                    &target_ledger_info,
                ) {
                    Ok(storage_service_request) => {
                        // Notify the peer of the missing data
                        match utils::notify_peer_of_new_data(
                            cached_storage_server_summary.clone(),
                            optimistic_fetches.clone(),
                            subscriptions.clone(),
                            lru_response_cache.clone(),
                            request_moderator.clone(),
                            storage.clone(),
                            time_service.clone(),
                            &peer,
                            storage_service_request,
                            target_ledger_info,
                            subscription_request.response_sender,
                        ) {
                            Ok(data_response) => {
                                // Update the subscription latency metric
                                let subscription_duration =
                                    time_service.now().duration_since(subscription_start_time);
                                metrics::observe_value_with_label(
                                    &metrics::SUBSCRIPTION_LATENCIES,
                                    peer.network_id(),
                                    &subscription_data_request.get_label(),
                                    subscription_duration.as_millis() as f64,
                                );

                                // Update the streams known version and epoch
                                if let Err(error) = subscription_stream_requests
                                    .update_known_version_and_epoch(&data_response)
                                {
                                    warn!(LogSchema::new(LogEntry::SubscriptionResponse).error(
                                        &Error::UnexpectedErrorEncountered(error.to_string())
                                    ));
                                }
                            },
                            Err(error) => {
                                warn!(LogSchema::new(LogEntry::SubscriptionResponse)
                                    .error(&Error::UnexpectedErrorEncountered(error.to_string())));
                            },
                        }
                    },
                    Err(error) => {
                        warn!(LogSchema::new(LogEntry::SubscriptionResponse)
                            .error(&Error::UnexpectedErrorEncountered(error.to_string())));
                    },
                }
            }
        }
    }

    Ok(())
}

/// Identifies the subscriptions that can be handled now.
/// Returns the list of peers that made those subscriptions
/// alongside the ledger info at the target version for the peer.
pub(crate) fn get_peers_with_ready_subscriptions<T: StorageReaderInterface>(
    cached_storage_server_summary: Arc<RwLock<StorageServerSummary>>,
    optimistic_fetches: Arc<Mutex<HashMap<PeerNetworkId, OptimisticFetchRequest>>>,
    subscriptions: Arc<Mutex<HashMap<PeerNetworkId, SubscriptionStreamRequests>>>,
    lru_response_cache: Arc<Mutex<LruCache<StorageServiceRequest, StorageServiceResponse>>>,
    request_moderator: Arc<RequestModerator>,
    storage: T,
    time_service: TimeService,
) -> aptos_storage_service_types::Result<Vec<(PeerNetworkId, LedgerInfoWithSignatures)>, Error> {
    // Fetch the latest storage summary and highest synced version
    let latest_storage_summary = cached_storage_server_summary.read().clone();
    let highest_synced_ledger_info = match latest_storage_summary.data_summary.synced_ledger_info {
        Some(ledger_info) => ledger_info,
        None => return Ok(vec![]),
    };
    let highest_synced_version = highest_synced_ledger_info.ledger_info().version();
    let highest_synced_epoch = highest_synced_ledger_info.ledger_info().epoch();

    // Identify the peers with ready subscriptions
    let mut ready_subscriptions = vec![];
    let mut invalid_peer_subscriptions = vec![];
    for (peer, subscription_stream_requests) in subscriptions.lock().iter_mut() {
        // Check that the first pending request is ready to be served
        if let Some(subscription_request) = subscription_stream_requests.first_pending_request() {
            if subscription_request.subscription_stream_index()
                == subscription_stream_requests.next_index_to_serve
            {
                // Check if there is new data to serve
                let highest_known_version = subscription_stream_requests.highest_known_version;
                if highest_known_version < highest_synced_version {
                    let highest_known_epoch = subscription_stream_requests.highest_known_epoch;
                    if highest_known_epoch < highest_synced_epoch {
                        // The peer needs to sync to their epoch ending ledger info
                        let epoch_ending_ledger_info = utils::get_epoch_ending_ledger_info(
                            cached_storage_server_summary.clone(),
                            optimistic_fetches.clone(),
                            subscriptions.clone(),
                            highest_known_epoch,
                            lru_response_cache.clone(),
                            request_moderator.clone(),
                            peer,
                            storage.clone(),
                            time_service.clone(),
                        )?;

                        // Check that we haven't been sent an subscription request
                        // (i.e., a request that does not respect an epoch boundary).
                        if epoch_ending_ledger_info.ledger_info().version() <= highest_known_version
                        {
                            invalid_peer_subscriptions.push(*peer);
                        } else {
                            ready_subscriptions.push((*peer, epoch_ending_ledger_info));
                        }
                    } else {
                        // The subscription is ready to be served
                        ready_subscriptions.push((*peer, highest_synced_ledger_info.clone()));
                    };
                }
            }
        }
    }

    // Remove the invalid subscriptions
    for peer in invalid_peer_subscriptions {
        if let Some(subscription_stream_requests) = subscriptions.lock().remove(&peer) {
            if let Some(subscription_request) = subscription_stream_requests.first_pending_request()
            {
                warn!(LogSchema::new(LogEntry::SubscriptionRefresh)
                    .error(&Error::InvalidRequest(
                        "Mismatch between known version and epoch!".into()
                    ))
                    .request(&subscription_request.request.clone())
                    .message("Dropping invalid subscription request!"));
            }
        }
    }

    // Return the ready subscriptions
    Ok(ready_subscriptions)
}

/// Removes all expired subscription streams
pub(crate) fn remove_expired_subscription_streams(
    config: StorageServiceConfig,
    subscriptions: Arc<Mutex<HashMap<PeerNetworkId, SubscriptionStreamRequests>>>,
) {
    // Remove expired subscription streams (i.e., streams that have been blocked for too long)
    subscriptions
        .lock()
        .retain(|peer_network_id, subscription_stream_requests| {
            // Update the expired subscription stream metrics
            if subscription_stream_requests.is_expired(config.max_subscription_period) {
                increment_counter(
                    &metrics::SUBSCRIPTION_EVENTS,
                    peer_network_id.network_id(),
                    SUBSCRIPTION_EXPIRE.into(),
                );
            }

            // Only retain non-expired subscription streams
            !subscription_stream_requests.is_expired(config.max_subscription_period)
        });
}
