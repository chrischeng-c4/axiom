//! Ported from Lib/test/test_iter_ported.py
//! Integration tests: builtins/iter.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_chain_two_iterables() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.chain([1, 2], [3, 4])))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_chain_three_iterables() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.chain([1, 2], [3, 4], [5])))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_islice_range() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.islice([0, 1, 2, 3, 4, 5], 2, 4)))
"#,
    );
    assert_output(&out, "[2, 3]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_product_two_iterables() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.product([1, 2], [3, 4])))
"#,
    );
    assert_output(&out, "[(1, 3), (1, 4), (2, 3), (2, 4)]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_combinations_pairs() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.combinations([1, 2, 3], 2)))
"#,
    );
    assert_output(&out, "[(1, 2), (1, 3), (2, 3)]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_combinations_with_replacement() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.combinations_with_replacement([1, 2, 3], 2)))
"#,
    );
    assert_output(&out, "[(1, 1), (1, 2), (1, 3), (2, 2), (2, 3), (3, 3)]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_permutations_full() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.permutations([1, 2, 3])))
"#,
    );
    assert_output(
        &out,
        "[(1, 2, 3), (1, 3, 2), (2, 1, 3), (2, 3, 1), (3, 1, 2), (3, 2, 1)]\n",
    );
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_accumulate_default_sum() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.accumulate([1, 2, 3, 4])))
"#,
    );
    assert_output(&out, "[1, 3, 6, 10]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_takewhile() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.takewhile(lambda x: x < 4, [1, 2, 3, 4, 5])))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_dropwhile() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.dropwhile(lambda x: x < 4, [1, 2, 3, 4, 5])))
"#,
    );
    assert_output(&out, "[4, 5]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_starmap() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.starmap(pow, [(2, 3), (3, 2)])))
"#,
    );
    assert_output(&out, "[8, 9]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_compress() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.compress(["a", "b", "c", "d"], [1, 0, 1, 1])))
"#,
    );
    assert_output(&out, "['a', 'c', 'd']\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_filterfalse() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.filterfalse(lambda x: x % 2, range(10))))
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_zip_longest_with_fillvalue() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.zip_longest([1, 2, 3], ["a", "b"], fillvalue="-")))
"#,
    );
    assert_output(&out, "[(1, 'a'), (2, 'b'), (3, '-')]\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_itertools_repeat_bounded() {
    let out = jit_capture(
        r#"import itertools
print(list(itertools.repeat("x", 3)))
"#,
    );
    assert_output(&out, "['x', 'x', 'x']\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_tuple_for_loop() {
    let out = jit_capture(
        r#"total = 0
for x in (10, 20, 30):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "60\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_range_for_loop() {
    let out = jit_capture(
        r#"total = 0
for i in range(5):
    total = total + i
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_range_with_start_stop() {
    let out = jit_capture(
        r#"total = 0
for i in range(2, 6):
    total = total + i
print(total)
"#,
    );
    assert_output(&out, "14\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_iter_next_list() {
    let out = jit_capture(
        r#"it = iter([10, 20, 30])
print(next(it))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "10\n20\n30\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_iter_next_tuple() {
    let out = jit_capture(
        r#"it = iter((1, 2, 3))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "1\n2\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_iter_next_str() {
    let out = jit_capture(
        r#"it = iter("xy")
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "x\ny\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_enumerate_with_start() {
    let out = jit_capture(
        r#"for i, x in enumerate(["a", "b"], 10):
    print(i, x)
"#,
    );
    assert_output(&out, "10 a\n11 b\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_zip_two_lists() {
    let out = jit_capture(
        r#"for a, b in zip([1, 2, 3], ["x", "y", "z"]):
    print(a, b)
"#,
    );
    assert_output(&out, "1 x\n2 y\n3 z\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_zip_three_sequences() {
    let out = jit_capture(
        r#"for a, b, c in zip([1, 2], [10, 20], [100, 200]):
    print(a + b + c)
"#,
    );
    assert_output(&out, "111\n222\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_zip_truncates_to_shortest() {
    let out = jit_capture(
        r#"for a, b in zip([1, 2, 3, 4], [10, 20]):
    print(a, b)
"#,
    );
    assert_output(&out, "1 10\n2 20\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_reversed_list() {
    let out = jit_capture(
        r#"for x in reversed([1, 2, 3]):
    print(x)
"#,
    );
    assert_output(&out, "3\n2\n1\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_reversed_tuple() {
    let out = jit_capture(
        r#"for x in reversed((10, 20, 30)):
    print(x)
"#,
    );
    assert_output(&out, "30\n20\n10\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_sum_over_range() {
    let out = jit_capture(
        r#"print(sum(range(11)))
"#,
    );
    assert_output(&out, "55\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_min_max_over_list() {
    let out = jit_capture(
        r#"print(min([5, 1, 3, 2, 4]))
print(max([5, 1, 3, 2, 4]))
"#,
    );
    assert_output(&out, "1\n5\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_nested_for_loop_sum() {
    let out = jit_capture(
        r#"total = 0
for i in range(3):
    for j in range(3):
        total = total + i * j
print(total)
"#,
    );
    assert_output(&out, "9\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_for_else_runs_when_no_break() {
    let out = jit_capture(
        r#"for x in [1, 2, 3]:
    print(x)
else:
    print("done")
"#,
    );
    assert_output(&out, "1\n2\n3\ndone\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_for_break_skips_else() {
    let out = jit_capture(
        r#"for x in [1, 2, 3]:
    if x == 2:
        break
    print(x)
else:
    print("done")
print("after")
"#,
    );
    assert_output(&out, "1\nafter\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_continue_skips_body_remainder() {
    let out = jit_capture(
        r#"for x in [1, 2, 3, 4]:
    if x == 2:
        continue
    print(x)
"#,
    );
    assert_output(&out, "1\n3\n4\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_next_exhausts_list() {
    let out = jit_capture(
        r#"it = iter([1, 2, 3])
print(next(it))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_next_raises_stop_iteration_when_drained() {
    let out = jit_capture(
        r#"it = iter([1])
print(next(it))
try:
    next(it)
except StopIteration:
    print("stopped")
"#,
    );
    assert_output(&out, "1\nstopped\n");
}

/// Ported from `Lib/test/test_iter_ported.py`.
#[test]
fn test_iter_over_string_yields_chars() {
    let out = jit_capture(
        r#"it = iter("abc")
print(next(it))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "a\nb\nc\n");
}

