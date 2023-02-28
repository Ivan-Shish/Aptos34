module aptos_std::ttt {
    struct SimpleMap {

    }
    use aptos_std::simple_map::make;

    #[test]
    fun test_for_each() {
        let m = make(1, 4, 2, 5);
        let s = 0;
        aptos_std::simple_map::for_each(m, |x, y| {
            s = s + x + y;
        });
        assert!(s == 12, 0)
    }

    #[test]
    fun test_for_each_ref() {
        let m = make(1, 4, 2, 5);
        let s = 0;
        aptos_std::simple_map::for_each_ref(&m, |x, y| {
            s = s + *x + *y;
        });
        assert!(s == 12, 0)
    }

    #[test]
    fun test_for_each_mut() {
        let m = make(1, 4, 2, 5);
        aptos_std::simple_map::for_each_mut(&mut m, |_key, val| {
            let val : &mut u64 = val;
            *val = *val + 1
        });
        assert!(*aptos_std::simple_map::borrow(&m, &1) == 5, 1)
    }

    #[test]
    fun test_fold() {
        let m = make(1, 4, 2, 5);
        let r = aptos_std::simple_map::fold(m, 0, |accu, key, val| {
            accu + key + val
        });
        assert!(r == 12, 0);
    }

    #[test]
    fun test_map() {
        use aptos_std::simple_map::borrow;
        let m = make(1, 4, 2, 5);
        let r = aptos_std::simple_map::map(m, |val| val + 1);
        assert!(*borrow(&r, &1) == 5, 1)
    }

    #[test]
    fun test_map_ref() {
        use aptos_std::simple_map::borrow;
        let m = make(1, 4, 2, 5);
        let r = aptos_std::simple_map::map_ref(&m, |val| *val + 1);
        assert!(*borrow(&r, &1) == 5, 1)
    }

    #[test]
    fun test_filter() {
        use aptos_std::simple_map::borrow;
        let m = make(1, 4, 2, 5);
        let r = aptos_std::simple_map::filter(m, |val| *val > 4);
        assert!(aptos_std::simple_map::length(&r) == 1, 1);
        assert!(*borrow(&r, &2) == 5, 1)
    }

    #[test]
    fun test_any() {
        let m = make(1, 4, 2, 5);
        let r = aptos_std::simple_map::any(&m, |_k, v| *v > 4);
        assert!(r, 1)
    }

    #[test]
    fun test_all() {
        let m = make(1, 4, 2, 5);
        let r = aptos_std::simple_map::all(&m, |_k, v| *v > 4);
        assert!(!r, 1)
    }
}
