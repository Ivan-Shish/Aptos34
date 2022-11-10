module aptos_std::iterable_table {
    use std::option::{Self, Option};
    use aptos_std::table::{Self, Table};

    /// The iterable wrapper around value, points to previous and next key if any.
    struct IterableValue<K: copy + store + drop, V: store> has store {
        val: V,
        prev: Option<K>,
        next: Option<K>,
    }

    /// An iterable table implementation based on double linked list.
    struct IterableTable<K: copy + store + drop, V: store> has store {
        inner: Table<K, IterableValue<K, V>>,
        head: Option<K>,
        tail: Option<K>,
    }

    /// Regular table API.

    /// Create an empty table.
    public fun new<K: copy + store + drop, V: store>(): IterableTable<K, V> {
        IterableTable {
            inner: table::new(),
            head: option::none(),
            tail: option::none(),
        }
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists.
    public fun add<K: copy + store + drop, V: store>(table: &mut IterableTable<K, V>, key: K, val: V) {
        let wrapped_value = IterableValue {
            val,
            prev: table.tail,
            next: option::none(),
        };
        table::add(&mut table.inner, key, wrapped_value);
        if (option::is_some(&table.tail)) {
            let k = option::borrow(&table.tail);
            table::borrow_mut(&mut table.inner, *k).next = option::some(key);
        } else {
            table.head = option::some(key);
        };
        table.tail = option::some(key);
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove<K: copy + store + drop, V: store>(table: &mut IterableTable<K, V>, key: K): V {
        let (val, _, _) = remove_iter(table, key);
        val
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow<K: copy + store + drop, V: store>(table: &IterableTable<K, V>, key: K): &V {
        &table::borrow(&table.inner, key).val
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut<K: copy + store + drop, V: store>(table: &mut IterableTable<K, V>, key: K): &mut V {
        &mut table::borrow_mut(&mut table.inner, key).val
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public fun borrow_mut_with_default<K: copy + store + drop, V: store + drop>(table: &mut IterableTable<K, V>, key: K, default: V): &mut V {
        if (!contains(table, key)) {
            add(table, key, default)
        };
        borrow_mut(table, key)
    }


    /// Returns true iff `table` contains an entry for `key`.
    public fun contains<K: copy + store + drop, V: store>(table: &IterableTable<K, V>, key: K): bool {
        table::contains(&table.inner, key)
    }

    /// Iterable API.

    /// Returns the key of the head for iteration.
    public fun head_key<K: copy + store + drop, V: store>(table: &IterableTable<K, V>): Option<K> {
        table.head
    }

    /// Returns the key of the tail for iteration.
    public fun tail_key<K: copy + store + drop, V: store>(table: &IterableTable<K, V>): Option<K> {
        table.tail
    }

    /// Acquire an immutable reference to the IterableValue which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_iter<K: copy + store + drop, V: store>(table: &IterableTable<K, V>, key: K): (&V, Option<K>, Option<K>) {
        let v = table::borrow(&table.inner, key);
        (&v.val, v.prev, v.next)
    }

    /// Acquire a mutable reference to the value and previous/next key which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_iter_mut<K: copy + store + drop, V: store>(table: &mut IterableTable<K, V>, key: K): (&mut V, Option<K>, Option<K>) {
        let v = table::borrow_mut(&mut table.inner, key);
        (&mut v.val, v.prev, v.next)
    }

    /// Remove from `table` and return the value and previous/next key which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove_iter<K: copy + store + drop, V: store>(table: &mut IterableTable<K, V>, key: K): (V, Option<K>, Option<K>) {
        let val = table::remove(&mut table.inner, copy key);
        if (option::contains(&table.tail, &key)) {
            table.tail = val.prev;
        };
        if (option::contains(&table.head, &key)) {
            table.head = val.next;
        };
        if (option::is_some(&val.prev)) {
            let key = option::borrow(&val.prev);
            table::borrow_mut(&mut table.inner, *key).next = val.next;
        };
        if (option::is_some(&val.next)) {
            let key = option::borrow(&val.next);
            table::borrow_mut(&mut table.inner, *key).prev = val.prev;
        };
        let IterableValue {val, prev, next} = val;
        (val, prev, next)
    }

    /// Remove all items from v2 and append to v1.
    public fun append<K: copy + store + drop, V: store>(v1: &mut IterableTable<K, V>, v2: &mut IterableTable<K, V>) {
        let key = head_key(v2);
        while (option::is_some(&key)) {
            let (val, _, next) = remove_iter(v2, *option::borrow(&key));
            add(v1, *option::borrow(&key), val);
            key = next;
        };
    }

    #[test_only]
    public fun destroy<K: copy + store + drop, V: store>(table: IterableTable<K, V>) {
        assert!(option::is_none(&table.head), 0);
        assert!(option::is_none(&table.tail), 0);
        let IterableTable {inner, head: _, tail: _} = table;
        table::drop_unchecked(inner);
    }

}
