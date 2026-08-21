//! Py3.12 conformance tests for set / frozenset (R3, issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_set.py — TestSet, TestFrozenSet, TestSetOfSets,
//!   TestBasicOps, TestMutatingOps
//!
//! Coverage (AC2): one positive + one error-case per R3 method:
//!   add, discard, remove, pop, clear, union, intersection, difference,
//!   symmetric_difference, and their operator forms.
//!   frozenset hashability and set-of-frozensets.
//!
//! Tests that require unimplemented features are marked `#[ignore]` with
//! a comment naming the missing behavior.
//!
//! @issue #759

use super::{assert_contains, assert_output, jit_capture};

// ── set.add ──────────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_add.
/// Positive: add a new element increases len by 1.
#[test]
fn test_set_add_positive() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.add(4)
print(len(s))
print(4 in s)
"#,
    );
    assert_output(&out, "4\nTrue\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_add (idempotent add).
/// add() is a no-op when element is already present.
#[test]
fn test_set_add_duplicate_is_noop() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.add(2)
print(len(s))
"#,
    );
    assert_output(&out, "3\n");
}

/// REQ: R3
/// Negative: add() returns None (matches CPython: always returns NoneType).
#[test]
fn test_set_add_returns_none() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set()
result = s.add(1)
print(result is None)
"#,
    );
    assert_output(&out, "True\n");
}

// ── set.discard ───────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_discard (element present).
#[test]
fn test_set_discard_present() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.discard(2)
print(len(s))
print(2 in s)
"#,
    );
    assert_output(&out, "2\nFalse\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_discard (element absent).
/// discard() does NOT raise KeyError when element is absent.
#[test]
fn test_set_discard_absent_no_error() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.discard(99)
print(len(s))
"#,
    );
    assert_output(&out, "3\n");
}

// ── set.remove ────────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_remove.
#[test]
fn test_set_remove_present() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.remove(2)
print(len(s))
print(2 in s)
"#,
    );
    assert_output(&out, "2\nFalse\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_remove_keyerror.
/// remove() raises KeyError when element is absent.
#[test]
fn test_set_remove_absent_raises_keyerror() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
try:
    s.remove(99)
except KeyError:
    print('KeyError')
"#,
    );
    assert_output(&out, "KeyError\n");
}

// ── set.pop ───────────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_pop.
/// pop() removes and returns an arbitrary element.
///
/// REQ: R1
#[test]
fn test_set_pop_positive() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
x = s.pop()
print(x in [1, 2, 3])
print(len(s))
"#,
    );
    assert_output(&out, "True\n2\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_pop_empty.
/// pop() raises KeyError when called on an empty set.
///
/// REQ: R1
#[test]
fn test_set_pop_empty_raises_keyerror() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set()
try:
    s.pop()
except KeyError:
    print('KeyError')
"#,
    );
    assert_output(&out, "KeyError\n");
}

// ── set.clear ─────────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestMutatingOps.test_clear.
#[test]
fn test_set_clear_positive() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
s.clear()
print(len(s))
"#,
    );
    assert_output(&out, "0\n");
}

/// REQ: R3
/// clear() on an already-empty set is a no-op (no error).
#[test]
fn test_set_clear_empty_noop() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set()
s.clear()
print(len(s))
"#,
    );
    assert_output(&out, "0\n");
}

// ── set.union / | ─────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBinaryOps.test_or.
#[test]
fn test_set_union_method() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {3, 4, 5}
result = a.union(b)
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5]\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestBinaryOps.test_or (operator form).
#[test]
fn test_set_union_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {3, 4, 5}
result = a | b
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5]\n");
}

/// REQ: R3
/// union with empty set returns copy of original (CPython invariant).
#[test]
fn test_set_union_with_empty() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
result = a | set()
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// REQ: R3
/// In-place union (|=) operator — desugars to a = a | b.
#[test]
fn test_set_union_inplace_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
a |= {3, 4}
print(sorted(a))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n");
}

// ── set.intersection / & ──────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBinaryOps.test_and.
#[test]
fn test_set_intersection_method() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
result = a.intersection(b)
print(sorted(result))
"#,
    );
    assert_output(&out, "[3, 4]\n");
}

/// REQ: R3
/// Operator form of intersection.
#[test]
fn test_set_intersection_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
result = a & b
print(sorted(result))
"#,
    );
    assert_output(&out, "[3, 4]\n");
}

/// REQ: R3
/// Intersection of disjoint sets is empty.
#[test]
fn test_set_intersection_disjoint() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
b = {3, 4}
result = a & b
print(len(result))
"#,
    );
    assert_output(&out, "0\n");
}

/// REQ: R3
/// In-place intersection (&=) — desugars to a = a & b.
#[test]
fn test_set_intersection_inplace_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
a &= {2, 3, 4}
print(sorted(a))
"#,
    );
    assert_output(&out, "[2, 3]\n");
}

// ── set.difference / - ────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBinaryOps.test_sub.
#[test]
fn test_set_difference_method() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
result = a.difference(b)
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2]\n");
}

/// REQ: R3
/// Operator form of difference.
#[test]
fn test_set_difference_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
result = a - b
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2]\n");
}

/// REQ: R3
/// Difference with empty set returns copy of original.
#[test]
fn test_set_difference_with_empty() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
result = a - set()
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// REQ: R3
/// In-place difference (-=) — desugars to a = a - b.
#[test]
fn test_set_difference_inplace_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
a -= {2}
print(sorted(a))
"#,
    );
    assert_output(&out, "[1, 3]\n");
}

// ── set.symmetric_difference / ^ ─────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBinaryOps.test_xor.
#[test]
fn test_set_symmetric_difference_method() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
result = a.symmetric_difference(b)
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 5, 6]\n");
}

/// REQ: R3
/// Operator form of symmetric_difference.
#[test]
fn test_set_symmetric_difference_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
result = a ^ b
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 5, 6]\n");
}

/// REQ: R3
/// Symmetric difference of identical sets is empty.
#[test]
fn test_set_symmetric_difference_identical() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {1, 2, 3}
result = a ^ b
print(len(result))
"#,
    );
    assert_output(&out, "0\n");
}

/// REQ: R3
/// In-place symmetric_difference (^=) — desugars to a = a ^ b.
#[test]
fn test_set_symmetric_difference_inplace_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
a ^= {3, 4}
print(sorted(a))
"#,
    );
    assert_output(&out, "[1, 2, 4]\n");
}

// ── set membership / in ───────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBasicOps.test_contains.
#[test]
fn test_set_contains_present() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
print(2 in s)
print(4 in s)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// REQ: R3
/// 'not in' operator (complement of 'in').
#[test]
fn test_set_not_contains() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
print(99 not in s)
"#,
    );
    assert_output(&out, "True\n");
}

// ── frozenset: basic ops ───────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestFrozenSet.test_constructor.
/// frozenset() from an iterable.
#[test]
fn test_frozenset_from_iterable() {
    // REQ: R3
    let out = jit_capture(
        r#"fs = frozenset([1, 2, 3, 2, 1])
print(sorted(fs))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// REQ: R3
/// frozenset() with no arguments creates empty frozenset.
#[test]
fn test_frozenset_empty() {
    // REQ: R3
    let out = jit_capture(
        r#"fs = frozenset()
print(len(fs))
"#,
    );
    assert_output(&out, "0\n");
}

/// REQ: R3
/// frozenset supports 'in' membership test.
#[test]
fn test_frozenset_contains() {
    // REQ: R3
    let out = jit_capture(
        r#"fs = frozenset([1, 2, 3])
print(1 in fs)
print(4 in fs)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// REQ: R3
/// frozenset: hash() is consistent across two calls on the same object.
/// CPython frozenset hash is content-based (XOR of element hashes).
/// Mamba uses pointer-based hash for frozenset — hash(fs) == hash(fs) holds
/// because it's the same object.
#[test]
fn test_frozenset_hash_self_consistency() {
    // REQ: R3
    let out = jit_capture(
        r#"fs = frozenset([1, 2, 3])
print(hash(fs) == hash(fs))
"#,
    );
    assert_output(&out, "True\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestFrozenSet.test_hash.
/// Two equal frozensets must have the same hash.
///
/// IGNORED: Mamba uses pointer-based hash for frozenset — two distinct but
/// equal frozenset objects return different hashes. CPython requires equal
/// objects to have equal hashes. Bug: frozenset hash is not content-based.
#[test]
fn test_frozenset_equal_objects_same_hash() {
    // REQ: R3
    let out = jit_capture(
        r#"fs1 = frozenset([1, 2, 3])
fs2 = frozenset([1, 2, 3])
print(hash(fs1) == hash(fs2))
"#,
    );
    assert_output(&out, "True\n");
}

/// REQ: R3
/// frozenset is immutable — mutating methods are not available.
#[test]
fn test_frozenset_no_add_method() {
    // REQ: R3
    let out = jit_capture(
        r#"fs = frozenset([1, 2])
try:
    fs.add(3)
except AttributeError:
    print('AttributeError')
"#,
    );
    assert_contains(&out, "AttributeError");
}

// ── set-of-frozensets ─────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestSetOfSets.
/// A set can contain frozensets as elements (frozenset is hashable).
///
/// IGNORED: Mamba's set uses linear-scan equality; frozenset elements in a set
/// require pointer-identity equality (since hash is pointer-based), so two
/// distinct frozenset([1,2]) objects will appear as separate elements even if
/// equal. The test expects 1 unique frozenset per distinct value group.
/// Additionally, set-of-frozensets requires frozenset content-based __hash__
/// which is currently not implemented.
#[test]
fn test_set_of_frozensets_len() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {frozenset([1, 2]), frozenset([3, 4])}
print(len(s))
"#,
    );
    assert_output(&out, "2\n");
}

// ── set equality ─────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBasicOps.test_eq.
#[test]
fn test_set_equality_equal() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {3, 1, 2}
print(a == b)
"#,
    );
    assert_output(&out, "True\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestBasicOps.test_ne.
#[test]
fn test_set_equality_not_equal() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {1, 2}
print(a == b)
print(a != b)
"#,
    );
    assert_output(&out, "False\nTrue\n");
}

// ── issubset / issuperset / isdisjoint ────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBinaryOps.test_issubset.
#[test]
fn test_set_issubset_true() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
b = {1, 2, 3}
print(a.issubset(b))
print(a <= b)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestBinaryOps.test_issuperset.
#[test]
fn test_set_issuperset_true() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3}
b = {1, 2}
print(a.issuperset(b))
print(a >= b)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// REQ: R3
/// isdisjoint: two sets with no common elements.
#[test]
fn test_set_isdisjoint_true() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
b = {3, 4}
print(a.isdisjoint(b))
"#,
    );
    assert_output(&out, "True\n");
}

/// REQ: R3
/// isdisjoint: two sets with a common element.
#[test]
fn test_set_isdisjoint_false() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
b = {2, 3}
print(a.isdisjoint(b))
"#,
    );
    assert_output(&out, "False\n");
}

// ── set constructor / len / iter ──────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestBasicOps.test_constructor.
/// set() with no args creates empty set.
#[test]
fn test_set_empty_constructor() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set()
print(len(s))
"#,
    );
    assert_output(&out, "0\n");
}

/// REQ: R3
/// set() deduplicates — ported from CPython test_set.py TestBasicOps.
#[test]
fn test_set_deduplication() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 1, 2, 2, 3}
print(len(s))
print(sorted(s))
"#,
    );
    assert_output(&out, "3\n[1, 2, 3]\n");
}

/// REQ: R3
/// Ported from CPython test_set.py TestBasicOps.test_iteration.
/// for-loop over a set visits all elements.
#[test]
fn test_set_iteration() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
result = []
for x in s:
    result.append(x)
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

// ── set comprehension ─────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py (comprehension construction).
#[test]
fn test_set_comprehension() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {x * x for x in range(5)}
print(sorted(s))
"#,
    );
    assert_output(&out, "[0, 1, 4, 9, 16]\n");
}

/// REQ: R3
/// Set comprehension with filter.
#[test]
fn test_set_comprehension_with_filter() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {x for x in range(10) if x % 2 == 0}
print(sorted(s))
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n");
}

// ── set update (method form of |=) ────────────────────────────────────────────

/// REQ: R3
/// CPython set.update(other) adds all elements from other to self.
/// Mamba dispatches update through dispatch_set_method.
///
/// REQ: R2
#[test]
fn test_set_update_method() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
a.update({3, 4})
print(sorted(a))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n");
}

// ── set issubset via <= / issuperset via >= ────────────────────────────────────

/// REQ: R4
/// Proper subset via < (strict subset).
#[test]
fn test_set_strict_subset_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
b = {1, 2, 3}
print(a < b)
print(b < a)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

// ── frozenset binary operations ───────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py TestFrozenSet.
/// frozenset supports intersection.
#[test]
fn test_frozenset_intersection() {
    // REQ: R3
    let out = jit_capture(
        r#"fs1 = frozenset([1, 2, 3])
fs2 = frozenset([2, 3, 4])
result = fs1 & fs2
print(sorted(result))
"#,
    );
    // frozenset & frozenset should return frozenset; print via sorted() (which accepts frozenset).
    // In Mamba, frozenset & frozenset calls mb_set_intersection which may return a set.
    // We only check the elements are correct (not the type).
    assert_output(&out, "[2, 3]\n");
}

/// REQ: R3
/// frozenset supports union.
#[test]
fn test_frozenset_union() {
    // REQ: R3
    let out = jit_capture(
        r#"fs1 = frozenset([1, 2])
fs2 = frozenset([2, 3])
result = fs1 | fs2
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// REQ: R3
/// frozenset supports difference.
#[test]
fn test_frozenset_difference() {
    // REQ: R3
    let out = jit_capture(
        r#"fs1 = frozenset([1, 2, 3])
fs2 = frozenset([2, 3])
result = fs1 - fs2
print(sorted(result))
"#,
    );
    assert_output(&out, "[1]\n");
}

// ── len / bool ───────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: empty set has len 0.
#[test]
fn test_set_len_empty() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set()
print(len(s))
"#,
    );
    assert_output(&out, "0\n");
}

/// REQ: R3
/// Ported from CPython test_set.py: bool(set()) is False, bool({1}) is True.
#[test]
fn test_set_bool_falsy_when_empty() {
    // REQ: R3
    let out = jit_capture(
        r#"print(bool(set()))
print(bool({1}))
"#,
    );
    assert_output(&out, "False\nTrue\n");
}

/// REQ: R3
/// Ported from CPython test_set.py: len(frozenset(...)) reflects unique elements.
#[test]
fn test_frozenset_len_with_duplicates() {
    // REQ: R3
    let out = jit_capture(
        r#"fs = frozenset([1, 1, 2, 2, 3])
print(len(fs))
"#,
    );
    assert_output(&out, "3\n");
}

// ── set construction ─────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: set() of a string iterates over chars.
#[test]
fn test_set_from_string_iterable() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set("abca")
print(sorted(s))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n");
}

/// REQ: R3
/// Ported from CPython test_set.py: set() of a tuple yields unique elements.
#[test]
fn test_set_from_tuple_iterable() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set((1, 2, 2, 3, 3, 3))
print(sorted(s))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

// ── copy semantics ───────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: set.copy() returns a new equal set.
#[test]
fn test_set_copy_is_equal_and_independent() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
t = s.copy()
print(s == t)
t.add(4)
print(sorted(s))
print(sorted(t))
"#,
    );
    assert_output(&out, "True\n[1, 2, 3]\n[1, 2, 3, 4]\n");
}

// ── frozenset.copy ───────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: frozenset.copy() returns an equal frozenset.
#[test]
fn test_frozenset_copy_is_equal() {
    // REQ: R3
    let out = jit_capture(
        r#"fs = frozenset([1, 2, 3])
ft = fs.copy()
print(fs == ft)
print(sorted(ft))
"#,
    );
    assert_output(&out, "True\n[1, 2, 3]\n");
}

// ── iteration / membership ───────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: `in` works after set construction from list.
#[test]
fn test_set_membership_after_list_construction() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set([10, 20, 30])
print(20 in s)
print(40 in s)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// REQ: R3
/// Ported from CPython test_set.py: iterating an empty set yields nothing.
#[test]
fn test_set_iterate_empty_yields_nothing() {
    // REQ: R3
    let out = jit_capture(
        r#"count = 0
for _ in set():
    count += 1
print(count)
"#,
    );
    assert_output(&out, "0\n");
}

// ── set & set, set | set, set - set with literal forms ───────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: chained union of three literal sets.
#[test]
fn test_set_union_three_way_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
b = {2, 3}
c = {3, 4}
result = a | b | c
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n");
}

/// REQ: R3
/// Ported from CPython test_set.py: chained intersection of three literal sets.
#[test]
fn test_set_intersection_three_way_operator() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2, 3, 4}
b = {2, 3, 4, 5}
c = {3, 4, 5, 6}
result = a & b & c
print(sorted(result))
"#,
    );
    assert_output(&out, "[3, 4]\n");
}

// ── set with mixed-type elements (int + str) ─────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: set can hold mixed hashable types.
#[test]
fn test_set_mixed_types_membership() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, "a", 2, "b"}
print(1 in s)
print("a" in s)
print(3 in s)
print("c" in s)
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\nFalse\n");
}

// ── cross set/frozenset interaction ──────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: frozenset equals set with same elements.
#[test]
fn test_set_equal_frozenset_with_same_elements() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2, 3}
fs = frozenset([1, 2, 3])
print(s == fs)
print(fs == s)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// REQ: R3
/// Ported from CPython test_set.py: set | frozenset returns a set with union.
#[test]
fn test_set_union_with_frozenset() {
    // REQ: R3
    let out = jit_capture(
        r#"s = {1, 2}
fs = frozenset([2, 3])
result = s | fs
print(sorted(result))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

// ── set.discard / set.remove on freshly constructed sets ─────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: discard on a constructed set then check membership.
#[test]
fn test_set_discard_then_membership() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set([1, 2, 3])
s.discard(2)
print(2 in s)
print(sorted(s))
"#,
    );
    assert_output(&out, "False\n[1, 3]\n");
}

// ── len semantics ────────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: len reflects elements after add/remove cycle.
#[test]
fn test_set_len_after_add_then_remove() {
    // REQ: R3
    let out = jit_capture(
        r#"s = set()
s.add(1)
s.add(2)
s.add(3)
s.remove(2)
print(len(s))
print(sorted(s))
"#,
    );
    assert_output(&out, "2\n[1, 3]\n");
}

// ── set inequality ───────────────────────────────────────────────────────────

/// REQ: R3
/// Ported from CPython test_set.py: != returns True for sets with different elements.
#[test]
fn test_set_not_equal_returns_true_for_different() {
    // REQ: R3
    let out = jit_capture(
        r#"a = {1, 2}
b = {1, 2, 3}
print(a != b)
print(b != a)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}
