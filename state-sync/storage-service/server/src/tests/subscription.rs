// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    moderator::RequestModerator,
    network::ResponseSender,
    storage::StorageReader,
    subscription,
    subscription::{SubscriptionRequest, SubscriptionStreamRequests},
    tests::{mock, utils},
};
use aptos_config::{config::StorageServiceConfig, network_id::PeerNetworkId};
use aptos_infallible::{Mutex, RwLock};
use aptos_storage_service_types::{
    requests::{
        DataRequest, StorageServiceRequest, SubscribeTransactionOutputsWithProofRequest,
        SubscribeTransactionsOrOutputsWithProofRequest, SubscribeTransactionsWithProofRequest,
    },
    responses::{CompleteDataRange, StorageServerSummary},
};
use aptos_time_service::TimeService;
use aptos_types::{epoch_change::EpochChangeProof, ledger_info::LedgerInfoWithSignatures};
use futures::channel::oneshot;
use lru::LruCache;
use rand::{rngs::OsRng, Rng};
use std::{collections::HashMap, sync::Arc};

#[tokio::test]
async fn test_peers_with_ready_subscriptions() {
    // Create a mock time service and subscriptions map
    let time_service = TimeService::mock();
    let subscriptions = Arc::new(Mutex::new(HashMap::new()));

    // Create three peers with ready subscriptions
    let mut peer_network_ids = vec![];
    for known_version in &[1, 5, 10] {
        // Create a random peer network id
        let peer_network_id = PeerNetworkId::random();
        peer_network_ids.push(peer_network_id);

        // Create a subscription stream and insert it into the pending map
        let subscription_stream_requests = create_subscription_stream_requests(
            time_service.clone(),
            Some(*known_version),
            Some(1),
            Some(0),
            Some(0),
        );
        subscriptions
            .lock()
            .insert(peer_network_id, subscription_stream_requests);
    }

    // Create epoch ending test data at version 9
    let epoch_ending_ledger_info = utils::create_epoch_ending_ledger_info(1, 9);
    let epoch_change_proof = EpochChangeProof {
        ledger_info_with_sigs: vec![epoch_ending_ledger_info],
        more: false,
    };

    // Create the mock db reader
    let mut db_reader = mock::create_mock_db_reader();
    utils::expect_get_epoch_ending_ledger_infos(&mut db_reader, 1, 2, epoch_change_proof);

    // Create the storage reader
    let storage_reader = StorageReader::new(StorageServiceConfig::default(), Arc::new(db_reader));

    // Create test data with an empty storage server summary
    let cached_storage_server_summary = Arc::new(RwLock::new(StorageServerSummary::default()));
    let optimistic_fetches = Arc::new(Mutex::new(HashMap::new()));
    let lru_response_cache = Arc::new(Mutex::new(LruCache::new(0)));
    let request_moderator = Arc::new(RequestModerator::new(
        cached_storage_server_summary.clone(),
        mock::create_peers_and_metadata(vec![]),
        StorageServiceConfig::default(),
        time_service.clone(),
    ));

    // Verify that there are no peers with ready subscriptions
    let peers_with_ready_subscriptions = subscription::get_peers_with_ready_subscriptions(
        cached_storage_server_summary.clone(),
        optimistic_fetches.clone(),
        subscriptions.clone(),
        lru_response_cache.clone(),
        request_moderator.clone(),
        storage_reader.clone(),
        time_service.clone(),
    )
    .unwrap();
    assert!(peers_with_ready_subscriptions.is_empty());

    // Update the storage server summary so that there is new data (at version 2)
    let highest_synced_ledger_info =
        update_storage_summary(2, 1, cached_storage_server_summary.clone());

    // Verify that peer 1 has a ready subscription
    let peers_with_ready_subscriptions = subscription::get_peers_with_ready_subscriptions(
        cached_storage_server_summary.clone(),
        optimistic_fetches.clone(),
        subscriptions.clone(),
        lru_response_cache.clone(),
        request_moderator.clone(),
        storage_reader.clone(),
        time_service.clone(),
    )
    .unwrap();
    assert_eq!(peers_with_ready_subscriptions, vec![(
        peer_network_ids[0],
        highest_synced_ledger_info
    )]);

    // Manually remove subscription 1 from the map
    subscriptions.lock().remove(&peer_network_ids[0]);

    // Update the storage server summary so that there is new data (at version 8)
    let highest_synced_ledger_info =
        update_storage_summary(8, 1, cached_storage_server_summary.clone());

    // Verify that peer 2 has a ready subscription
    let peers_with_ready_subscriptions = subscription::get_peers_with_ready_subscriptions(
        cached_storage_server_summary.clone(),
        optimistic_fetches.clone(),
        subscriptions.clone(),
        lru_response_cache.clone(),
        request_moderator.clone(),
        storage_reader.clone(),
        time_service.clone(),
    )
    .unwrap();
    assert_eq!(peers_with_ready_subscriptions, vec![(
        peer_network_ids[1],
        highest_synced_ledger_info
    )]);

    // Manually remove subscription 2 from the map
    subscriptions.lock().remove(&peer_network_ids[1]);

    // Update the storage server summary so that there is new data (at version 100)
    let _ = update_storage_summary(100, 2, cached_storage_server_summary.clone());

    // Verify that subscription 3 is not returned because it was invalid
    let peers_with_ready_subscriptions = subscription::get_peers_with_ready_subscriptions(
        cached_storage_server_summary,
        optimistic_fetches,
        subscriptions.clone(),
        lru_response_cache,
        request_moderator,
        storage_reader,
        time_service,
    )
    .unwrap();
    assert_eq!(peers_with_ready_subscriptions, vec![]);

    // Verify that the subscriptions are now empty
    assert!(subscriptions.lock().is_empty());
}

#[tokio::test]
async fn test_remove_expired_subscriptions_no_new_data() {
    // Create a storage service config
    let max_subscription_period_ms = 100;
    let storage_service_config = StorageServiceConfig {
        max_subscription_period_ms,
        ..Default::default()
    };

    // Create a mock time service
    let time_service = TimeService::mock();

    // Create the first batch of test subscriptions
    let num_subscriptions_in_batch = 10;
    let subscriptions = Arc::new(Mutex::new(HashMap::new()));
    for _ in 0..num_subscriptions_in_batch {
        let subscription_stream_requests =
            create_subscription_stream_requests(time_service.clone(), None, None, None, None);
        subscriptions
            .lock()
            .insert(PeerNetworkId::random(), subscription_stream_requests);
    }

    // Verify the number of active subscriptions
    assert_eq!(subscriptions.lock().len(), num_subscriptions_in_batch);

    // Elapse a small amount of time (not enough to expire the subscriptions)
    utils::elapse_time(max_subscription_period_ms / 2, &time_service).await;

    // Remove the expired subscriptions and verify none were removed
    subscription::remove_expired_subscription_streams(
        storage_service_config,
        subscriptions.clone(),
    );
    assert_eq!(subscriptions.lock().len(), num_subscriptions_in_batch);

    // Create another batch of test subscriptions
    for _ in 0..num_subscriptions_in_batch {
        let subscription_stream_requests =
            create_subscription_stream_requests(time_service.clone(), None, None, None, None);
        subscriptions
            .lock()
            .insert(PeerNetworkId::random(), subscription_stream_requests);
    }

    // Verify the new number of active subscriptions
    assert_eq!(subscriptions.lock().len(), num_subscriptions_in_batch * 2);

    // Elapse enough time to expire the first batch of subscriptions
    utils::elapse_time(max_subscription_period_ms, &time_service).await;

    // Remove the expired subscriptions and verify the first batch was removed
    subscription::remove_expired_subscription_streams(
        storage_service_config,
        subscriptions.clone(),
    );
    assert_eq!(subscriptions.lock().len(), num_subscriptions_in_batch);

    // Elapse enough time to expire the second batch of subscriptions
    utils::elapse_time(max_subscription_period_ms, &time_service).await;

    // Remove the expired subscriptions and verify the second batch was removed
    subscription::remove_expired_subscription_streams(
        storage_service_config,
        subscriptions.clone(),
    );
    assert!(subscriptions.lock().is_empty());
}

#[tokio::test]
async fn test_remove_expired_subscriptions_blocked_stream() {
    // Create a storage service config
    let max_subscription_period_ms = 100;
    let storage_service_config = StorageServiceConfig {
        max_subscription_period_ms,
        ..Default::default()
    };

    // Create a mock time service
    let time_service = TimeService::mock();

    // Create a batch of test subscriptions
    let num_subscriptions_in_batch = 10;
    let subscriptions = Arc::new(Mutex::new(HashMap::new()));
    let mut peer_network_ids = vec![];
    for i in 0..num_subscriptions_in_batch {
        // Create a new peer
        let peer_network_id = PeerNetworkId::random();
        peer_network_ids.push(peer_network_id);

        // Create a subscription stream request for the peer
        let subscription_stream_requests = create_subscription_stream_requests(
            time_service.clone(),
            Some(1),
            Some(1),
            Some(i as u64),
            Some(0),
        );
        subscriptions
            .lock()
            .insert(peer_network_id, subscription_stream_requests);
    }

    // Create test data with an empty storage server summary
    let cached_storage_server_summary = Arc::new(RwLock::new(StorageServerSummary::default()));
    let optimistic_fetches = Arc::new(Mutex::new(HashMap::new()));
    let lru_response_cache = Arc::new(Mutex::new(LruCache::new(0)));
    let request_moderator = Arc::new(RequestModerator::new(
        cached_storage_server_summary.clone(),
        mock::create_peers_and_metadata(vec![]),
        storage_service_config,
        time_service.clone(),
    ));
    let storage_reader = StorageReader::new(
        storage_service_config,
        Arc::new(mock::create_mock_db_reader()),
    );

    // Update the storage server summary so that there is new data (at version 5)
    let _ = update_storage_summary(5, 1, cached_storage_server_summary.clone());

    // Handle the active subscriptions
    subscription::handle_active_subscriptions(
        cached_storage_server_summary.clone(),
        storage_service_config,
        lru_response_cache.clone(),
        optimistic_fetches.clone(),
        request_moderator.clone(),
        storage_reader.clone(),
        subscriptions.clone(),
        time_service.clone(),
    )
    .unwrap();

    // Verify that all subscription streams are now empty because
    // the pending requests were sent.
    assert_eq!(subscriptions.lock().len(), num_subscriptions_in_batch);
    for (_, subscription_stream_requests) in subscriptions.lock().iter() {
        assert!(subscription_stream_requests
            .first_pending_request()
            .is_none());
    }

    // Elapse enough time to expire the blocked streams
    utils::elapse_time(max_subscription_period_ms + 1, &time_service).await;

    // Add a new subscription request to the first subscription stream
    let subscription_request =
        create_subscription_request(&time_service, Some(1), Some(1), Some(0), Some(1));
    add_subscription_request_to_stream(
        subscription_request,
        subscriptions.clone(),
        &peer_network_ids[0],
    );

    // Remove the expired subscriptions and verify all but one were removed
    subscription::remove_expired_subscription_streams(
        storage_service_config,
        subscriptions.clone(),
    );
    assert_eq!(subscriptions.lock().len(), 1);
    assert!(subscriptions.lock().contains_key(&peer_network_ids[0]));
}

#[tokio::test]
async fn test_remove_expired_subscriptions_blocked_stream_index() {
    // Create a storage service config
    let max_subscription_period_ms = 100;
    let storage_service_config = StorageServiceConfig {
        max_subscription_period_ms,
        ..Default::default()
    };

    // Create a mock time service
    let time_service = TimeService::mock();

    // Create the first batch of test subscriptions
    let num_subscriptions_in_batch = 10;
    let subscriptions = Arc::new(Mutex::new(HashMap::new()));
    for _ in 0..num_subscriptions_in_batch {
        let subscription_stream_requests = create_subscription_stream_requests(
            time_service.clone(),
            Some(1),
            Some(1),
            None,
            Some(0),
        );
        subscriptions
            .lock()
            .insert(PeerNetworkId::random(), subscription_stream_requests);
    }

    // Create test data with an empty storage server summary
    let cached_storage_server_summary = Arc::new(RwLock::new(StorageServerSummary::default()));
    let optimistic_fetches = Arc::new(Mutex::new(HashMap::new()));
    let lru_response_cache = Arc::new(Mutex::new(LruCache::new(0)));
    let request_moderator = Arc::new(RequestModerator::new(
        cached_storage_server_summary.clone(),
        mock::create_peers_and_metadata(vec![]),
        storage_service_config,
        time_service.clone(),
    ));
    let storage_reader = StorageReader::new(
        storage_service_config,
        Arc::new(mock::create_mock_db_reader()),
    );

    // Update the storage server summary so that there is new data (at version 5)
    let highest_synced_ledger_info =
        update_storage_summary(5, 1, cached_storage_server_summary.clone());

    // Verify that all peers have ready subscriptions (but don't serve them!)
    let peers_with_ready_subscriptions = subscription::get_peers_with_ready_subscriptions(
        cached_storage_server_summary.clone(),
        optimistic_fetches.clone(),
        subscriptions.clone(),
        lru_response_cache.clone(),
        request_moderator.clone(),
        storage_reader.clone(),
        time_service.clone(),
    )
    .unwrap();
    assert_eq!(
        peers_with_ready_subscriptions.len(),
        num_subscriptions_in_batch
    );

    // Elapse enough time to expire the subscriptions
    utils::elapse_time(max_subscription_period_ms + 1, &time_service).await;

    // Remove the expired subscriptions and verify they were all removed
    subscription::remove_expired_subscription_streams(
        storage_service_config,
        subscriptions.clone(),
    );
    assert!(subscriptions.lock().is_empty());

    // Create another batch of test subscriptions where the stream is
    // blocked on the next index to serve.
    let mut peer_network_ids = vec![];
    for i in 0..num_subscriptions_in_batch {
        // Create a new peer
        let peer_network_id = PeerNetworkId::random();
        peer_network_ids.push(peer_network_id);

        // Create a subscription stream request for the peer
        let subscription_stream_requests = create_subscription_stream_requests(
            time_service.clone(),
            Some(1),
            Some(1),
            None,
            Some(i as u64 + 1),
        );
        subscriptions
            .lock()
            .insert(peer_network_id, subscription_stream_requests);
    }

    // Verify the number of active subscriptions
    assert_eq!(subscriptions.lock().len(), num_subscriptions_in_batch);

    // Verify that none of the subscriptions are ready to be served (they are blocked)
    let peers_with_ready_subscriptions = subscription::get_peers_with_ready_subscriptions(
        cached_storage_server_summary.clone(),
        optimistic_fetches.clone(),
        subscriptions.clone(),
        lru_response_cache.clone(),
        request_moderator.clone(),
        storage_reader.clone(),
        time_service.clone(),
    )
    .unwrap();
    assert!(peers_with_ready_subscriptions.is_empty());

    // Elapse enough time to expire the batch of subscriptions
    utils::elapse_time(max_subscription_period_ms + 1, &time_service).await;

    // Add a new subscription request to the first subscription stream (to unblock it)
    let subscription_request =
        create_subscription_request(&time_service, Some(1), Some(1), None, Some(0));
    add_subscription_request_to_stream(
        subscription_request,
        subscriptions.clone(),
        &peer_network_ids[0],
    );

    // Verify that the first peer subscription stream is unblocked
    let peers_with_ready_subscriptions = subscription::get_peers_with_ready_subscriptions(
        cached_storage_server_summary.clone(),
        optimistic_fetches.clone(),
        subscriptions.clone(),
        lru_response_cache.clone(),
        request_moderator.clone(),
        storage_reader.clone(),
        time_service.clone(),
    )
    .unwrap();
    assert_eq!(peers_with_ready_subscriptions.len(), 1);
    assert!(
        peers_with_ready_subscriptions.contains(&(peer_network_ids[0], highest_synced_ledger_info))
    );

    // Remove the expired subscriptions and verify all but one were removed
    subscription::remove_expired_subscription_streams(
        storage_service_config,
        subscriptions.clone(),
    );
    assert_eq!(subscriptions.lock().len(), 1);
    assert!(subscriptions.lock().contains_key(&peer_network_ids[0]));
}

/// Adds a subscription request to the subscription stream for the given peer
fn add_subscription_request_to_stream(
    subscription_request: SubscriptionRequest,
    subscriptions: Arc<Mutex<HashMap<PeerNetworkId, SubscriptionStreamRequests>>>,
    peer_network_id: &PeerNetworkId,
) {
    let mut subscriptions = subscriptions.lock();
    let subscription_stream_requests = subscriptions.get_mut(peer_network_id).unwrap();
    subscription_stream_requests
        .add_subscription_request(subscription_request)
        .unwrap();
}

/// Creates a random request for subscription data
fn create_subscription_data_request(
    known_version: Option<u64>,
    known_epoch: Option<u64>,
    subscription_stream_id: Option<u64>,
    subscription_stream_index: Option<u64>,
) -> DataRequest {
    // Get the request data
    let known_version = known_version.unwrap_or_default();
    let known_epoch = known_epoch.unwrap_or_default();
    let subscription_stream_id = subscription_stream_id.unwrap_or_default();
    let subscription_stream_index = subscription_stream_index.unwrap_or_default();

    // Generate the random data request
    let mut rng = OsRng;
    let random_number: u8 = rng.gen();
    match random_number % 3 {
        0 => DataRequest::SubscribeTransactionOutputsWithProof(
            SubscribeTransactionOutputsWithProofRequest {
                known_version,
                known_epoch,
                subscription_stream_id,
                subscription_stream_index,
            },
        ),
        1 => DataRequest::SubscribeTransactionsWithProof(SubscribeTransactionsWithProofRequest {
            known_version,
            known_epoch,
            include_events: false,
            subscription_stream_id,
            subscription_stream_index,
        }),
        2 => DataRequest::SubscribeTransactionsOrOutputsWithProof(
            SubscribeTransactionsOrOutputsWithProofRequest {
                known_version,
                known_epoch,
                include_events: false,
                max_num_output_reductions: 0,
                subscription_stream_id,
                subscription_stream_index,
            },
        ),
        number => panic!("This shouldn't be possible! Got: {:?}", number),
    }
}

/// Creates a random subscription request using the given data
fn create_subscription_request(
    time_service: &TimeService,
    known_version: Option<u64>,
    known_epoch: Option<u64>,
    subscription_stream_id: Option<u64>,
    subscription_stream_index: Option<u64>,
) -> SubscriptionRequest {
    // Create a storage service request
    let data_request = create_subscription_data_request(
        known_version,
        known_epoch,
        subscription_stream_id,
        subscription_stream_index,
    );
    let storage_service_request = StorageServiceRequest::new(data_request, true);

    // Create the response sender
    let (callback, _) = oneshot::channel();
    let response_sender = ResponseSender::new(callback);

    // Create a subscription request
    SubscriptionRequest::new(
        storage_service_request,
        response_sender,
        time_service.clone(),
    )
}

/// Creates a random subscription stream using the given data
fn create_subscription_stream_requests(
    time_service: TimeService,
    known_version: Option<u64>,
    known_epoch: Option<u64>,
    subscription_stream_id: Option<u64>,
    subscription_stream_index: Option<u64>,
) -> SubscriptionStreamRequests {
    // Create a new subscription request
    let subscription_request = create_subscription_request(
        &time_service,
        known_version,
        known_epoch,
        subscription_stream_id,
        subscription_stream_index,
    );

    // Create and return the subscription stream containing the request
    SubscriptionStreamRequests::new(subscription_request, time_service)
}

/// Updates the storage server summary with new data and returns
/// the highest synced ledger info.
fn update_storage_summary(
    highest_version: u64,
    highest_epoch: u64,
    cached_storage_server_summary: Arc<RwLock<StorageServerSummary>>,
) -> LedgerInfoWithSignatures {
    // Create a new storage server summary
    let mut storage_server_summary = StorageServerSummary::default();

    // Update the highest synced ledger info
    let synced_ledger_info =
        utils::create_test_ledger_info_with_sigs(highest_epoch, highest_version);
    storage_server_summary.data_summary.synced_ledger_info = Some(synced_ledger_info.clone());

    // Update the epoch ending ledger info range
    storage_server_summary
        .data_summary
        .epoch_ending_ledger_infos = Some(CompleteDataRange::new(0, highest_epoch).unwrap());

    // Update the cached storage server summary
    *cached_storage_server_summary.write() = storage_server_summary;

    // Return the highest synced ledger info
    synced_ledger_info
}
