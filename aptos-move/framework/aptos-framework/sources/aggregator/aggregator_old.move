module aptos_framework::aggregator_old {
    use std::error;
    use std::option;
    use aptos_std::iterable_table::{Self, IterableTable};
    use aptos_framework::transaction_context;

    const EAGGREGATOR_OLD_ERROR: u64 = 17;

    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    struct AggregatorOld has store {
        buckets: IterableTable<u64, u128>
    }

    public fun new(): AggregatorOld {
       AggregatorOld { buckets: iterable_table::new() }
    }

    /// Adds a new value to the aggregator by storing it in one of the buckets.
    public fun add(aggregator: &mut AggregatorOld, value: u128) {
        let idx = transaction_context::get_bucket();

        if (iterable_table::contains(&aggregator.buckets, idx)) {
            let amount = iterable_table::borrow_mut(&mut aggregator.buckets, idx);
            assert!(*amount <= MAX_U128 - value, error::invalid_argument(EAGGREGATOR_OLD_ERROR));
            *amount = *amount + value;
        } else {
            iterable_table::add(&mut aggregator.buckets, idx, value);
        }
    }

    public fun drain(aggregator: &AggregatorOld): u128 {
        let amount = 0;

        let key = iterable_table::head_key(&aggregator.buckets);
        while (option::is_some(&key)) {
            let (value, _, next) = iterable_table::borrow_iter(&aggregator.buckets, *option::borrow(&key));
            key = next;
            assert!(amount <= MAX_U128 - *value, error::invalid_argument(EAGGREGATOR_OLD_ERROR));
            amount = amount + *value;
        };
        amount
    }

    #[test_only]
    fun destroy(aggregator: AggregatorOld) {
        let AggregatorOld { buckets } = aggregator;
        iterable_table::destroy(buckets);
    }

    #[test]
    fun aggregator_test() {
        let agg = new();

        add(&mut agg, 10);
        add(&mut agg, 10);
        add(&mut agg, 11);

        assert!(drain(&mut aggr) == 31, 0);

        destroy(agg);
    }
}
