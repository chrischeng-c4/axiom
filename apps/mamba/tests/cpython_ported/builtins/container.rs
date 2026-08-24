//! Ported from Lib/test/test_container_ported.py
//! Integration tests: builtins/container.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_sorted_ascending() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 4, 1, 5, 9, 2, 6]))
"#,
    );
    assert_output(&out, "[1, 1, 2, 3, 4, 5, 6, 9]\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_string_basic_and_open_slices() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[2:5])
print(s[:3])
print(s[5:])
print(s[-3:])
"#,
    );
    assert_output(&out, "cde\nabc\nfgh\nfgh\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_string_step_and_reverse() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[::2])
print(s[::-1])
"#,
    );
    assert_output(&out, "aceg\nhgfedcba\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_indexing_and_len() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(t[0])
print(t[-1])
print(t[1])
print(len(t))
"#,
    );
    assert_output(&out, "1\n3\n2\n3\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_concat_repeat_and_membership() {
    let out = jit_capture(
        r#"print((1, 2) + (3, 4))
print((1,) * 3)
print(1 in (1, 2, 3))
print(4 in (1, 2, 3))
"#,
    );
    assert_output(&out, "(1, 2, 3, 4)\n(1, 1, 1)\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_comparison_lexicographic() {
    let out = jit_capture(
        r#"print((1, 2) == (1, 2))
print((1, 2) < (1, 3))
print((1, 2) < (2, 0))
print((1, 2, 3) > (1, 2))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_constructor_and_singleton_empty() {
    let out = jit_capture(
        r#"print(tuple([1, 2, 3]))
print(tuple())
print((42,))
print(())
"#,
    );
    assert_output(&out, "(1, 2, 3)\n()\n(42,)\n()\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_unpack_into_variables() {
    let out = jit_capture(
        r#"a, b, c = (1, 2, 3)
print(a)
print(b)
print(c)
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_abs_int() {
    let out = jit_capture(
        r#"print(abs(5))
print(abs(-5))
print(abs(0))
"#,
    );
    assert_output(&out, "5\n5\n0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_all_true_cases() {
    let out = jit_capture(
        r#"print(all([1, 2, 3]))
print(all([True, True, True]))
print(all([]))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_all_false_cases() {
    let out = jit_capture(
        r#"print(all([1, 0, 3]))
print(all([False, True]))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_any_true_cases() {
    let out = jit_capture(
        r#"print(any([0, 0, 1]))
print(any([False, True]))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_any_false_cases() {
    let out = jit_capture(
        r#"print(any([0, 0, 0]))
print(any([False, False]))
print(any([]))
"#,
    );
    assert_output(&out, "False\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_bin() {
    let out = jit_capture(
        r#"print(bin(5))
print(bin(0))
print(bin(255))
"#,
    );
    assert_output(&out, "0b101\n0b0\n0b11111111\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_hex() {
    let out = jit_capture(
        r#"print(hex(255))
print(hex(0))
print(hex(16))
"#,
    );
    assert_output(&out, "0xff\n0x0\n0x10\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_oct() {
    let out = jit_capture(
        r#"print(oct(8))
print(oct(0))
print(oct(64))
"#,
    );
    assert_output(&out, "0o10\n0o0\n0o100\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_chr_ord_roundtrip() {
    let out = jit_capture(
        r#"print(ord(chr(100)))
print(chr(ord("Z")))
"#,
    );
    assert_output(&out, "100\nZ\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_divmod_int() {
    let out = jit_capture(
        r#"print(divmod(17, 5))
print(divmod(10, 3))
"#,
    );
    assert_output(&out, "(3, 2)\n(3, 1)\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_isinstance_int() {
    let out = jit_capture(
        r#"print(isinstance(5, int))
print(isinstance("x", int))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_isinstance_str() {
    let out = jit_capture(
        r#"print(isinstance("hi", str))
print(isinstance(5, str))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_isinstance_list() {
    let out = jit_capture(
        r#"print(isinstance([1, 2], list))
print(isinstance((1, 2), list))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_len_sequences() {
    let out = jit_capture(
        r#"print(len("hello"))
print(len([1, 2, 3]))
print(len((1, 2, 3, 4)))
print(len({1: "a", 2: "b"}))
print(len({1, 2, 3}))
print(len(b"hello"))
"#,
    );
    assert_output(&out, "5\n3\n4\n2\n3\n5\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_max_variadic() {
    let out = jit_capture(
        r#"print(max(1, 5, 3))
print(max(10, 20))
"#,
    );
    assert_output(&out, "5\n20\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_max_iterable() {
    let out = jit_capture(
        r#"print(max([3, 1, 4, 1, 5, 9, 2, 6]))
print(max((10, 20, 30)))
"#,
    );
    assert_output(&out, "9\n30\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_min_variadic() {
    let out = jit_capture(
        r#"print(min(5, 1, 3))
print(min(10, 20))
"#,
    );
    assert_output(&out, "1\n10\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_min_iterable() {
    let out = jit_capture(
        r#"print(min([3, 1, 4, 1, 5]))
print(min((30, 20, 10)))
"#,
    );
    assert_output(&out, "1\n10\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_pow_two_arg_int() {
    let out = jit_capture(
        r#"print(pow(2, 10))
print(pow(3, 4))
print(pow(7, 0))
"#,
    );
    assert_output(&out, "1024\n81\n1\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_pow_three_arg_int() {
    let out = jit_capture(
        r#"print(pow(2, 10, 1000))
print(pow(3, 5, 7))
"#,
    );
    assert_output(&out, "24\n5\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_repr_int_str() {
    let out = jit_capture(
        r#"print(repr(42))
print(repr("hello"))
"#,
    );
    assert_output(&out, "42\n'hello'\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_round_int_arg() {
    let out = jit_capture(
        r#"print(round(3.7))
print(round(3.4))
print(round(-3.7))
print(round(0.5))
"#,
    );
    assert_output(&out, "4\n3\n-4\n0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_round_ndigits() {
    let out = jit_capture(
        r#"print(round(3.14159, 2))
print(round(1.005, 2))
"#,
    );
    assert_output(&out, "3.14\n1.0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_sum_iterable() {
    let out = jit_capture(
        r#"print(sum([1, 2, 3, 4]))
print(sum((10, 20, 30)))
print(sum([]))
"#,
    );
    assert_output(&out, "10\n60\n0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_sum_with_start() {
    let out = jit_capture(
        r#"print(sum([1, 2, 3], 10))
print(sum([], 100))
"#,
    );
    assert_output(&out, "16\n100\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_sorted_reverse() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 4, 1, 5], reverse=True))
"#,
    );
    assert_output(&out, "[5, 4, 3, 1, 1]\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_builtin_sorted_strings() {
    let out = jit_capture(
        r#"print(sorted(["banana", "apple", "cherry"]))
"#,
    );
    assert_output(&out, "['apple', 'banana', 'cherry']\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_index_len_slice() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(t)
print(t[0], t[-1])
print(len(t))
print(t[1:4])
"#,
    );
    assert_output(&out, "(1, 2, 3, 4, 5)\n1 5\n5\n(2, 3, 4)\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_reductions_and_membership() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4, 5)
print(sum(t))
print(min(t), max(t))
print(2 in t)
print(99 in t)
print(t.count(3))
"#,
    );
    assert_output(&out, "15\n1 5\nTrue\nFalse\n1\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_concat_and_repeat() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(t + (4, 5))
print(t * 2)
print((0,) + t)
print(() + t)
print(("x",) * 4)
"#,
    );
    assert_output(
        &out,
        "(1, 2, 3, 4, 5)\n(1, 2, 3, 1, 2, 3)\n(0, 1, 2, 3)\n(1, 2, 3)\n('x', 'x', 'x', 'x')\n",
    );
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_divmod_returns_quotient_remainder() {
    let out = jit_capture(
        r#"print(divmod(10, 3))
print(divmod(20, 4))
print(divmod(7, 2))
"#,
    );
    assert_output(&out, "(3, 1)\n(5, 0)\n(3, 1)\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_abs_on_int_and_float() {
    let out = jit_capture(
        r#"print(abs(-5))
print(abs(5))
print(abs(-3.5))
print(abs(0))
"#,
    );
    assert_output(&out, "5\n5\n3.5\n0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_pow_two_argument_form() {
    let out = jit_capture(
        r#"print(pow(2, 10))
print(pow(3, 4))
print(pow(5, 0))
"#,
    );
    assert_output(&out, "1024\n81\n1\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_min_max_varargs_and_iterable() {
    let out = jit_capture(
        r#"print(min(3, 1, 4))
print(max(3, 1, 4))
print(min([5, 2, 8]))
print(max([5, 2, 8]))
"#,
    );
    assert_output(&out, "1\n4\n2\n8\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_empty_literal_len() {
    let out = jit_capture(
        r#"t = ()
print(len(t))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_literal_len() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(len(t))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_singleton_requires_comma() {
    let out = jit_capture(
        r#"t = (5,)
print(len(t))
print(t[0])
"#,
    );
    assert_output(&out, "1\n5\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_constructor_empty() {
    let out = jit_capture(
        r#"t = tuple()
print(len(t))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_constructor_from_list() {
    let out = jit_capture(
        r#"t = tuple([1, 2, 3])
print(len(t))
print(t[0])
print(t[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_getitem_positive() {
    let out = jit_capture(
        r#"t = (10, 20, 30)
print(t[0])
print(t[1])
print(t[2])
"#,
    );
    assert_output(&out, "10\n20\n30\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_getitem_negative() {
    let out = jit_capture(
        r#"t = (10, 20, 30)
print(t[-1])
print(t[-2])
"#,
    );
    assert_output(&out, "30\n20\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_iterate_sum() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4)
total = 0
for x in t:
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_iterate_empty_yields_nothing() {
    let out = jit_capture(
        r#"t = ()
count = 0
for x in t:
    count = count + 1
print(count)
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_equal_same_elements() {
    let out = jit_capture(
        r#"a = (1, 2, 3)
b = (1, 2, 3)
print(a == b)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_not_equal_different_order() {
    let out = jit_capture(
        r#"a = (1, 2, 3)
b = (3, 2, 1)
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_not_equal_different_len() {
    let out = jit_capture(
        r#"a = (1, 2, 3)
b = (1, 2)
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_bool_empty_is_false() {
    let out = jit_capture(
        r#"t = ()
print(bool(t))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"t = (0,)
print(bool(t))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_concatenation_operator() {
    let out = jit_capture(
        r#"a = (1, 2)
b = (3, 4)
c = a + b
print(len(c))
print(c[0])
print(c[3])
"#,
    );
    assert_output(&out, "4\n1\n4\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_repetition_operator() {
    let out = jit_capture(
        r#"t = (1, 2) * 3
print(len(t))
print(t[0])
print(t[5])
"#,
    );
    assert_output(&out, "6\n1\n2\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_contains_present() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(2 in t)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_contains_absent() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(99 in t)
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_count_present() {
    let out = jit_capture(
        r#"t = (1, 2, 2, 3, 2)
print(t.count(2))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_count_absent_is_zero() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(t.count(99))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_index_present() {
    let out = jit_capture(
        r#"t = (10, 20, 30)
print(t.index(20))
"#,
    );
    assert_output(&out, "1\n");
}

/// Ported from `Lib/test/test_container_ported.py`.
#[test]
fn test_tuple_nested_access() {
    let out = jit_capture(
        r#"t = ((1, 2), (3, 4))
print(t[0][1])
print(t[1][0])
"#,
    );
    assert_output(&out, "2\n3\n");
}

