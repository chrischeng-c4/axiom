//! Ported from Lib/test/test_list_ported.py
//! Integration tests: builtins/list.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_bool_constructor_from_list_empty() {
    let out = jit_capture(
        r#"print(bool([]))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_bool_constructor_from_list_nonempty() {
    let out = jit_capture(
        r#"print(bool([0]))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_slice_with_step() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6]
print(xs[2:5])
print(xs[::2])
print(xs[::-1])
"#,
    );
    assert_output(&out, "[2, 3, 4]\n[0, 2, 4, 6]\n[6, 5, 4, 3, 2, 1, 0]\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_index_first_occurrence() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 2, 1]
print(xs.index(2))
print(xs.index(3))
print([10, 20, 30].index(30))
"#,
    );
    assert_output(&out, "1\n2\n2\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_count_present_and_absent() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 2, 1]
print(xs.count(1))
print(xs.count(2))
print(xs.count(99))
print([].count("x"))
"#,
    );
    assert_output(&out, "2\n2\n0\n0\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_reverse_in_place_and_builtin_reversed() {
    let out = jit_capture(
        r#"ys = [1, 2, 3]
ys.reverse()
print(ys)
zs = list(reversed([1, 2, 3]))
print(zs)
print(list(reversed("abc")))
"#,
    );
    assert_output(&out, "[3, 2, 1]\n[3, 2, 1]\n['c', 'b', 'a']\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_iter_list_for_loop() {
    let out = jit_capture(
        r#"total = 0
for x in [1, 2, 3, 4]:
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_and_tuple_constructors() {
    let out = jit_capture(
        r#"print(list("abc"))
print(tuple([1, 2, 3]))
print(list((4, 5, 6)))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n(1, 2, 3)\n[4, 5, 6]\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_basic_and_open_ended_slices() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(xs[2:5])
print(xs[:4])
print(xs[6:])
print(xs[:])
"#,
    );
    assert_output(
        &out,
        "[2, 3, 4]\n[0, 1, 2, 3]\n[6, 7, 8, 9]\n[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n",
    );
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_step_slices_including_reverse() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(xs[::2])
print(xs[1::2])
print(xs[::-1])
"#,
    );
    assert_output(
        &out,
        "[0, 2, 4, 6, 8]\n[1, 3, 5, 7, 9]\n[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]\n",
    );
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_negative_index_slices() {
    let out = jit_capture(
        r#"xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(xs[-3:])
print(xs[-5:-2])
print(xs[:-5])
"#,
    );
    assert_output(&out, "[7, 8, 9]\n[5, 6, 7]\n[0, 1, 2, 3, 4]\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_empty_literal_len() {
    let out = jit_capture(
        r#"xs = []
print(len(xs))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_literal_len() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(len(xs))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_constructor_empty() {
    let out = jit_capture(
        r#"xs = list()
print(len(xs))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_constructor_from_iterable() {
    let out = jit_capture(
        r#"xs = list((1, 2, 3))
print(len(xs))
print(xs[0])
print(xs[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_getitem_positive() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
print(xs[0])
print(xs[1])
print(xs[2])
"#,
    );
    assert_output(&out, "10\n20\n30\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_getitem_negative() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
print(xs[-1])
print(xs[-2])
"#,
    );
    assert_output(&out, "30\n20\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_setitem() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
xs[1] = 99
print(xs[0])
print(xs[1])
print(xs[2])
"#,
    );
    assert_output(&out, "1\n99\n3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_append_increases_len() {
    let out = jit_capture(
        r#"xs = [1, 2]
xs.append(3)
print(len(xs))
print(xs[2])
"#,
    );
    assert_output(&out, "3\n3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_extend_with_list() {
    let out = jit_capture(
        r#"xs = [1, 2]
xs.extend([3, 4])
print(len(xs))
print(xs[3])
"#,
    );
    assert_output(&out, "4\n4\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_insert_at_front() {
    let out = jit_capture(
        r#"xs = [2, 3]
xs.insert(0, 1)
print(len(xs))
print(xs[0])
print(xs[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_pop_default_last() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
v = xs.pop()
print(v)
print(len(xs))
"#,
    );
    assert_output(&out, "3\n2\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_pop_indexed() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
v = xs.pop(0)
print(v)
print(len(xs))
print(xs[0])
"#,
    );
    assert_output(&out, "10\n2\n20\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_remove_present() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 2]
xs.remove(2)
print(len(xs))
print(xs[1])
"#,
    );
    assert_output(&out, "3\n3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_slice_basic() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
ys = xs[1:4]
print(len(ys))
print(ys[0])
print(ys[2])
"#,
    );
    assert_output(&out, "3\n2\n4\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_slice_open_start() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
ys = xs[:3]
print(len(ys))
print(ys[0])
print(ys[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_slice_open_end() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
ys = xs[2:]
print(len(ys))
print(ys[0])
"#,
    );
    assert_output(&out, "3\n3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_iterate_sum() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4]
total = 0
for x in xs:
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_iterate_empty_yields_nothing() {
    let out = jit_capture(
        r#"xs = []
count = 0
for x in xs:
    count = count + 1
print(count)
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_equal_same_elements() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = [1, 2, 3]
print(a == b)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_not_equal_different_order() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = [3, 2, 1]
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_not_equal_different_len() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = [1, 2]
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_bool_empty_is_false() {
    let out = jit_capture(
        r#"xs = []
print(bool(xs))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"xs = [0]
print(bool(xs))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_count_present() {
    let out = jit_capture(
        r#"xs = [1, 2, 2, 3, 2]
print(xs.count(2))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_count_absent_is_zero() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(xs.count(99))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_index_present() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
print(xs.index(20))
"#,
    );
    assert_output(&out, "1\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_concatenation_operator() {
    let out = jit_capture(
        r#"a = [1, 2]
b = [3, 4]
c = a + b
print(len(c))
print(c[0])
print(c[3])
"#,
    );
    assert_output(&out, "4\n1\n4\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_repetition_operator() {
    let out = jit_capture(
        r#"xs = [1, 2] * 3
print(len(xs))
print(xs[0])
print(xs[5])
"#,
    );
    assert_output(&out, "6\n1\n2\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_contains_present() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(2 in xs)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_contains_absent() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(99 in xs)
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_append_extend_insert() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
xs.append(4)
print(xs)
xs.extend([5, 6])
print(xs)
xs.insert(0, 0)
print(xs)
"#,
    );
    assert_output(
        &out,
        "[1, 2, 3, 4]\n[1, 2, 3, 4, 5, 6]\n[0, 1, 2, 3, 4, 5, 6]\n",
    );
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_remove_and_pop() {
    let out = jit_capture(
        r#"xs = [10, 20, 30, 40]
xs.remove(20)
print(xs)
last = xs.pop()
print(last)
print(xs)
"#,
    );
    assert_output(&out, "[10, 30, 40]\n40\n[10, 30]\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_list_sort_in_place() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
xs.sort()
print(xs)
"#,
    );
    assert_output(&out, "[1, 1, 2, 3, 4, 5, 6, 9]\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_sorted_builtin_with_reverse() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 2]))
print(sorted([3, 1, 2], reverse=True))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[3, 2, 1]\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_append_extend_insert() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
xs.append(4)
print(xs)
xs.extend([5, 6])
print(xs)
xs.insert(0, 0)
print(xs)
xs.insert(3, 99)
print(xs)
"#,
    );
    assert_output(
        &out,
        "[1, 2, 3, 4]\n[1, 2, 3, 4, 5, 6]\n[0, 1, 2, 3, 4, 5, 6]\n[0, 1, 2, 99, 3, 4, 5, 6]\n",
    );
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_remove_and_pop_variants() {
    let out = jit_capture(
        r#"xs = [10, 20, 30, 40, 50]
xs.remove(30)
print(xs)
last = xs.pop()
print(last)
print(xs)
first = xs.pop(0)
print(first)
print(xs)
"#,
    );
    assert_output(&out, "[10, 20, 40, 50]\n50\n[10, 20, 40]\n10\n[20, 40]\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_reverse_sort_clear() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
xs.reverse()
print(xs)
xs.sort()
print(xs)
xs.sort(reverse=True)
print(xs)
xs.clear()
print(xs)
print(len(xs))
"#,
    );
    assert_output(
        &out,
        "[6, 2, 9, 5, 1, 4, 1, 3]\n[1, 1, 2, 3, 4, 5, 6, 9]\n[9, 6, 5, 4, 3, 2, 1, 1]\n[]\n0\n",
    );
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_transform_and_filter() {
    let out = jit_capture(
        r#"words = ["apple", "bee", "cat", "donkey", "elephant"]
print([len(w) for w in words])
print([w.upper() for w in words])
print([w for w in words if len(w) > 3])
"#,
    );
    assert_output(
        &out,
        "[5, 3, 3, 6, 8]\n['APPLE', 'BEE', 'CAT', 'DONKEY', 'ELEPHANT']\n['apple', 'donkey', 'elephant']\n",
    );
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_projection_to_tuple() {
    let out = jit_capture(
        r#"words = ["apple", "bee", "cat"]
print([(w, len(w)) for w in words])
print([(i, w) for i, w in enumerate(words)])
"#,
    );
    assert_output(
        &out,
        "[('apple', 5), ('bee', 3), ('cat', 3)]\n[(0, 'apple'), (1, 'bee'), (2, 'cat')]\n",
    );
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_arithmetic_filter_and_sum_genexp() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4, 5]
print([n * n for n in nums])
print([n for n in nums if n % 2 == 0])
print(sum(n for n in nums))
"#,
    );
    assert_output(&out, "[1, 4, 9, 16, 25]\n[2, 4]\n15\n");
}

/// Ported from `Lib/test/test_list_ported.py`.
#[test]
fn test_range_to_list_start_stop() {
    let out = jit_capture(
        r#"xs = list(range(2, 6))
print(xs)
"#,
    );
    assert_output(&out, "[2, 3, 4, 5]\n");
}

/// REQ: R3
/// Ported from `Lib/test/test_list_ported.py`.
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

