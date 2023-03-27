// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use aptos_metrics_core::{
    exponential_buckets, register_histogram_vec, HistogramVec,
};
use once_cell::sync::Lazy;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub static OTHER_TIMERS_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "misc_timers_seconds",
        // metric description
        "Misc timers.",
        // metric labels (dimensions)
        &["name"],
        exponential_buckets(/*start=*/ 1e-6, /*factor=*/ 2.0, /*count=*/ 22).unwrap(),
    )
        .unwrap()
});
