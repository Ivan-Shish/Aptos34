
<a name="0x1_aggregator_old"></a>

# Module `0x1::aggregator_old`



-  [Struct `AggregatorOld`](#0x1_aggregator_old_AggregatorOld)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x1_aggregator_old_new)
-  [Function `add`](#0x1_aggregator_old_add)
-  [Function `drain`](#0x1_aggregator_old_drain)


<pre><code><b>use</b> <a href="../../aptos-stdlib/../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table">0x1::iterable_table</a>;
<b>use</b> <a href="../../aptos-stdlib/../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="transaction_context.md#0x1_transaction_context">0x1::transaction_context</a>;
</code></pre>



<a name="0x1_aggregator_old_AggregatorOld"></a>

## Struct `AggregatorOld`



<pre><code><b>struct</b> <a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">AggregatorOld</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>buckets: <a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table_IterableTable">iterable_table::IterableTable</a>&lt;u64, u128&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_aggregator_old_MAX_U128"></a>



<pre><code><b>const</b> <a href="aggregator_old.md#0x1_aggregator_old_MAX_U128">MAX_U128</a>: u128 = 340282366920938463463374607431768211455;
</code></pre>



<a name="0x1_aggregator_old_EAGGREGATOR_OLD_ERROR"></a>



<pre><code><b>const</b> <a href="aggregator_old.md#0x1_aggregator_old_EAGGREGATOR_OLD_ERROR">EAGGREGATOR_OLD_ERROR</a>: u64 = 17;
</code></pre>



<a name="0x1_aggregator_old_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="aggregator_old.md#0x1_aggregator_old_new">new</a>(): <a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">aggregator_old::AggregatorOld</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="aggregator_old.md#0x1_aggregator_old_new">new</a>(): <a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">AggregatorOld</a> {
   <a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">AggregatorOld</a> { buckets: <a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table_new">iterable_table::new</a>() }
}
</code></pre>



</details>

<a name="0x1_aggregator_old_add"></a>

## Function `add`

Adds a new value to the aggregator by storing it in one of the buckets.


<pre><code><b>public</b> <b>fun</b> <a href="aggregator_old.md#0x1_aggregator_old_add">add</a>(<a href="aggregator.md#0x1_aggregator">aggregator</a>: &<b>mut</b> <a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">aggregator_old::AggregatorOld</a>, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="aggregator_old.md#0x1_aggregator_old_add">add</a>(<a href="aggregator.md#0x1_aggregator">aggregator</a>: &<b>mut</b> <a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">AggregatorOld</a>, value: u128) {
    <b>let</b> idx = <a href="transaction_context.md#0x1_transaction_context_get_bucket">transaction_context::get_bucket</a>();

    <b>if</b> (<a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table_contains">iterable_table::contains</a>(&<a href="aggregator.md#0x1_aggregator">aggregator</a>.buckets, idx)) {
        <b>let</b> amount = <a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table_borrow_mut">iterable_table::borrow_mut</a>(&<b>mut</b> <a href="aggregator.md#0x1_aggregator">aggregator</a>.buckets, idx);
        <b>assert</b>!(*amount &lt;= <a href="aggregator_old.md#0x1_aggregator_old_MAX_U128">MAX_U128</a> - value, <a href="../../aptos-stdlib/../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="aggregator_old.md#0x1_aggregator_old_EAGGREGATOR_OLD_ERROR">EAGGREGATOR_OLD_ERROR</a>));
        *amount = *amount + value;
    } <b>else</b> {
        <a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table_add">iterable_table::add</a>(&<b>mut</b> <a href="aggregator.md#0x1_aggregator">aggregator</a>.buckets, idx, value);
    }
}
</code></pre>



</details>

<a name="0x1_aggregator_old_drain"></a>

## Function `drain`



<pre><code><b>public</b> <b>fun</b> <a href="aggregator_old.md#0x1_aggregator_old_drain">drain</a>(<a href="aggregator.md#0x1_aggregator">aggregator</a>: &<a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">aggregator_old::AggregatorOld</a>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="aggregator_old.md#0x1_aggregator_old_drain">drain</a>(<a href="aggregator.md#0x1_aggregator">aggregator</a>: &<a href="aggregator_old.md#0x1_aggregator_old_AggregatorOld">AggregatorOld</a>): u128 {
    <b>let</b> amount = 0;

    <b>let</b> key = <a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table_head_key">iterable_table::head_key</a>(&<a href="aggregator.md#0x1_aggregator">aggregator</a>.buckets);
    <b>while</b> (<a href="../../aptos-stdlib/../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&key)) {
        <b>let</b> (value, _, next) = <a href="../../aptos-stdlib/doc/iterable_table.md#0x1_iterable_table_borrow_iter">iterable_table::borrow_iter</a>(&<a href="aggregator.md#0x1_aggregator">aggregator</a>.buckets, *<a href="../../aptos-stdlib/../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(&key));
        key = next;
        <b>assert</b>!(amount &lt;= <a href="aggregator_old.md#0x1_aggregator_old_MAX_U128">MAX_U128</a> - *value, <a href="../../aptos-stdlib/../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="aggregator_old.md#0x1_aggregator_old_EAGGREGATOR_OLD_ERROR">EAGGREGATOR_OLD_ERROR</a>));
        amount = amount + *value;
    };
    amount
}
</code></pre>



</details>


[move-book]: https://move-language.github.io/move/introduction.html
