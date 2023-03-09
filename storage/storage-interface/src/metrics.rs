// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use aptos_metrics_core::{
    exponential_buckets, register_histogram_vec, register_int_counter_vec, HistogramVec,
    IntCounterVec,
};
use once_cell::sync::Lazy;

pub static STATE_VIEW_CACHE_HIT_EVENT: &str = "cache_hit";
pub static STATE_VIEW_CACHE_MISS_EVENT: &str = "cache_miss";

pub static TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "aptos_storage_interface_timer_seconds",
        "Various timers for performance analysis.",
        &["name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
    .unwrap()
});

pub static STATE_VIEW_CACHE_EVENT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "state_view_cache_event",
        "Cache hit for cached state view",
        &["event", "caller"]
    )
    .unwrap()
});
