// This file is created to verify the vector module in the standard library.
// This file is basically a clone of `stdlib/modules/vector.move` with renaming the module and function names.
// In this file, the functions with prefix of `verify_model` are verifying the corresponding built-in Boogie
// procedures that they inline (e.g., `verify_model_remove`).
// This file also verifies the actual Move implementations of non-native functions (e.g., `verify_remove`).
module 0x42::VerifyVector {
    use std::vector;

    public fun verify_model_rotate<Element>(
        v: &mut vector<Element>,
        rot: u64
    ): u64 {
        vector::rotate(v, rot)
    }
    spec verify_model_rotate {
        aborts_if rot > len(v);
        ensures old(len(v)) == len(v);
        ensures result == len(v) - rot;
        ensures forall i in 0..len(v) - rot: v[i] == old(v[i + rot]);
        ensures forall i in 0..rot: old(v[i]) == v[len(v) - rot + i];
    }

    public fun verify_model_rotate_slice<Element>(
        v: &mut vector<Element>,
        left: u64,
        rot: u64,
        right: u64
    ): u64 {
        vector::rotate_slice(v, left, rot, right)
    }
    spec verify_model_rotate_slice {
        aborts_if right > len(v);
        aborts_if !(left <= rot && rot <= right);
        ensures old(len(v)) == len(v);
        ensures result == left + (right - rot);
        ensures forall i in 0..left: v[i] == old(v[i]);
        ensures forall i in right..len(v): v[i] == old(v[i]);
        ensures forall i in left..left + (right - rot): v[i] == old(v[i + rot - left]);
        ensures forall i in left..rot - left: old(v[i]) == v[right - rot + i];
    }

    public fun verify_model_insert<Element>(v: &mut vector<Element>, l: u64, e: Element) {
        vector::insert(v, l, e)
    }
    spec verify_model_insert {
        aborts_if l > len(v);
        ensures len(v) == old(len(v)) + 1;
        ensures forall i in 0..l: old(v[i]) == v[i];
        ensures v[l] == e;
        ensures forall i in l..old(len(v)): old(v[i]) == v[i + 1];
    }

    // Reverses the order of the elements in the vector in place.
    public fun verify_model_reverse_slice<Element>(v: &mut vector<Element>, l: u64, r: u64) {
        vector::reverse_slice(v, l, r)
    }
    spec verify_model_reverse_slice {
        aborts_if l > r;
        aborts_if l < r && r > len(v);
        ensures len(v) == old(len(v));
        ensures forall i in 0..l: old(v[i]) == v[i];
        ensures forall i in r..len(v): old(v[i]) == v[i];
        ensures forall i in l..r: old(v[i]) == v[r + l - i - 1];
    }


    // Moves all of the elements of the `other` vector into the `lhs` vector.
    public fun verify_model_reverse_append<Element>(lhs: &mut vector<Element>, other: vector<Element>) {
        vector::reverse_append(lhs, other); // inlining the built-in Boogie procedure
    }
    spec verify_model_reverse_append {
        ensures len(lhs) == old(len(lhs) + len(other));
        ensures lhs[0..len(old(lhs))] == old(lhs);
        ensures forall i in 0..len(lhs) - len(old(lhs)): lhs[len(lhs) - i - 1] == other[i];
    }

    public fun verify_model_trim<Element>(v: &mut vector<Element>, new_len: u64): vector<Element> {
        vector::trim(v, new_len)
    }
    spec verify_model_trim {
        aborts_if new_len > len(v);
        ensures len(v) == new_len;
        ensures v[0..new_len] == old(v[0..new_len]);
        ensures result == old(v[new_len..len((v))]);
    }

    public fun verify_model_trim_reverse<Element>(v: &mut vector<Element>, new_len: u64): vector<Element> {
        vector::trim_reverse(v, new_len)
    }
    spec verify_model_trim_reverse {
        aborts_if new_len > len(v);
        ensures len(v) == new_len;
        ensures v[0..new_len] == old(v[0..new_len]);
        ensures forall i in 0..len(old(v)) - new_len: old(v[len(v) - i - 1]) == result[i];
    }
}
