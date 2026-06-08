//! Py3.12 conformance tests for the `itertools` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_itertools.py):
//!   list-aliased surface — chain, islice, product, permutations,
//!   combinations, combinations_with_replacement, accumulate, takewhile,
//!   dropwhile, starmap, compress, filterfalse, zip_longest, bounded repeat.
//!
//! Unbounded iterators (`count`, `cycle`) and the few that yield infinite
//! results are excluded — they require a real `MbObject::Generator`
//! variant (cross-cutting gap #2182) and currently raise
//! `TypeError: object is not an iterator`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ---------------------------------------------------------------- chain / islice

#[test]
fn test_itertools_chain_two_iterables() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.chain([1, 2], [3, 4])))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n");
}

#[test]
fn test_itertools_chain_three_iterables() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.chain([1, 2], [3, 4], [5])))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5]\n");
}

#[test]
fn test_itertools_islice_range() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.islice([0, 1, 2, 3, 4, 5], 2, 4)))
"#,
    );
    assert_output(&out, "[2, 3]\n");
}

// ---------------------------------------------------------------- product

#[test]
fn test_itertools_product_two_iterables() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.product([1, 2], [3, 4])))
"#,
    );
    assert_output(&out, "[(1, 3), (1, 4), (2, 3), (2, 4)]\n");
}

// ---------------------------------------------------------------- combinations / permutations

#[test]
fn test_itertools_combinations_pairs() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.combinations([1, 2, 3], 2)))
"#,
    );
    assert_output(&out, "[(1, 2), (1, 3), (2, 3)]\n");
}

#[test]
fn test_itertools_combinations_with_replacement() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.combinations_with_replacement([1, 2, 3], 2)))
"#,
    );
    assert_output(&out, "[(1, 1), (1, 2), (1, 3), (2, 2), (2, 3), (3, 3)]\n");
}

#[test]
fn test_itertools_permutations_full() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.permutations([1, 2, 3])))
"#,
    );
    assert_output(&out, "[(1, 2, 3), (1, 3, 2), (2, 1, 3), (2, 3, 1), (3, 1, 2), (3, 2, 1)]\n");
}

// ---------------------------------------------------------------- accumulate

#[test]
fn test_itertools_accumulate_default_sum() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.accumulate([1, 2, 3, 4])))
"#,
    );
    assert_output(&out, "[1, 3, 6, 10]\n");
}

// ---------------------------------------------------------------- takewhile / dropwhile

#[test]
fn test_itertools_takewhile() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.takewhile(lambda x: x < 4, [1, 2, 3, 4, 5])))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

#[test]
fn test_itertools_dropwhile() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.dropwhile(lambda x: x < 4, [1, 2, 3, 4, 5])))
"#,
    );
    assert_output(&out, "[4, 5]\n");
}

// ---------------------------------------------------------------- starmap

#[test]
fn test_itertools_starmap() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.starmap(pow, [(2, 3), (3, 2)])))
"#,
    );
    assert_output(&out, "[8, 9]\n");
}

// ---------------------------------------------------------------- compress / filterfalse

#[test]
fn test_itertools_compress() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.compress(["a", "b", "c", "d"], [1, 0, 1, 1])))
"#,
    );
    assert_output(&out, "['a', 'c', 'd']\n");
}

#[test]
fn test_itertools_filterfalse() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.filterfalse(lambda x: x % 2, range(10))))
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n");
}

// ---------------------------------------------------------------- zip_longest

#[test]
fn test_itertools_zip_longest_with_fillvalue() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.zip_longest([1, 2, 3], ["a", "b"], fillvalue="-")))
"#,
    );
    assert_output(&out, "[(1, 'a'), (2, 'b'), (3, '-')]\n");
}

// ---------------------------------------------------------------- repeat (bounded)

#[test]
fn test_itertools_repeat_bounded() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.repeat("x", 3)))
"#,
    );
    assert_output(&out, "['x', 'x', 'x']\n");
}
