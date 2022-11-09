// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use aptos_transaction_benchmarks::{
    config, measurement::wall_time_measurement, transactions::TransactionBencher,
};
use criterion::{criterion_group, criterion_main, measurement::Measurement, Criterion};
use language_e2e_tests::account_universe::P2PTransferGen;
use proptest::prelude::*;

//
// Transaction benchmarks
//

fn peer_to_peer<M: Measurement + 'static>(c: &mut Criterion<M>) {
    c.bench_function("peer_to_peer", |b| {
        let bencher = TransactionBencher::new(any_with::<P2PTransferGen>((
            config::MIN_TRANSFER_AMOUNT,
            config::MAX_TRANSFER_AMOUNT,
        )));
        bencher.bench(b)
    });

    c.bench_function("peer_to_peer_parallel", |b| {
        let bencher = TransactionBencher::new(any_with::<P2PTransferGen>((
            config::MIN_TRANSFER_AMOUNT,
            config::MAX_TRANSFER_AMOUNT,
        )));
        bencher.bench_parallel(b)
    });
}

criterion_group!(
    name = txn_benches;
    config = wall_time_measurement().sample_size(10);
    targets = peer_to_peer
);

criterion_main!(txn_benches);
