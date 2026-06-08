//! Py3.12 conformance tests for builtins (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_builtin.py — BuiltinTest
//!
//! Coverage: abs, all, any, bin, hex, oct, chr, ord, divmod, isinstance,
//! len, max/min (variadic + sequence), pow (2- and 3-arg), repr, round,
//! sum (with start), sorted (ascending + reverse).
//!
//! @issue #759

use super::{assert_output, jit_capture};

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

#[test]
fn test_builtin_all_false_cases() {
    let out = jit_capture(
        r#"print(all([1, 0, 3]))
print(all([False, True]))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

#[test]
fn test_builtin_any_true_cases() {
    let out = jit_capture(
        r#"print(any([0, 0, 1]))
print(any([False, True]))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

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

#[test]
fn test_builtin_chr() {
    let out = jit_capture(
        r#"print(chr(65))
print(chr(97))
print(chr(48))
"#,
    );
    assert_output(&out, "A\na\n0\n");
}

#[test]
fn test_builtin_ord_str() {
    let out = jit_capture(
        r#"print(ord("A"))
print(ord("a"))
print(ord("0"))
"#,
    );
    assert_output(&out, "65\n97\n48\n");
}

#[test]
fn test_builtin_chr_ord_roundtrip() {
    let out = jit_capture(
        r#"print(ord(chr(100)))
print(chr(ord("Z")))
"#,
    );
    assert_output(&out, "100\nZ\n");
}

#[test]
fn test_builtin_divmod_int() {
    let out = jit_capture(
        r#"print(divmod(17, 5))
print(divmod(10, 3))
"#,
    );
    assert_output(&out, "(3, 2)\n(3, 1)\n");
}

#[test]
fn test_builtin_isinstance_int() {
    let out = jit_capture(
        r#"print(isinstance(5, int))
print(isinstance("x", int))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_builtin_isinstance_str() {
    let out = jit_capture(
        r#"print(isinstance("hi", str))
print(isinstance(5, str))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_builtin_isinstance_list() {
    let out = jit_capture(
        r#"print(isinstance([1, 2], list))
print(isinstance((1, 2), list))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

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

#[test]
fn test_builtin_max_variadic() {
    let out = jit_capture(
        r#"print(max(1, 5, 3))
print(max(10, 20))
"#,
    );
    assert_output(&out, "5\n20\n");
}

#[test]
fn test_builtin_max_iterable() {
    let out = jit_capture(
        r#"print(max([3, 1, 4, 1, 5, 9, 2, 6]))
print(max((10, 20, 30)))
"#,
    );
    assert_output(&out, "9\n30\n");
}

#[test]
fn test_builtin_min_variadic() {
    let out = jit_capture(
        r#"print(min(5, 1, 3))
print(min(10, 20))
"#,
    );
    assert_output(&out, "1\n10\n");
}

#[test]
fn test_builtin_min_iterable() {
    let out = jit_capture(
        r#"print(min([3, 1, 4, 1, 5]))
print(min((30, 20, 10)))
"#,
    );
    assert_output(&out, "1\n10\n");
}

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

#[test]
fn test_builtin_pow_three_arg_int() {
    let out = jit_capture(
        r#"print(pow(2, 10, 1000))
print(pow(3, 5, 7))
"#,
    );
    assert_output(&out, "24\n5\n");
}

#[test]
fn test_builtin_repr_int_str() {
    let out = jit_capture(
        r#"print(repr(42))
print(repr("hello"))
"#,
    );
    assert_output(&out, "42\n'hello'\n");
}

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

#[test]
fn test_builtin_round_ndigits() {
    let out = jit_capture(
        r#"print(round(3.14159, 2))
print(round(1.005, 2))
"#,
    );
    assert_output(&out, "3.14\n1.0\n");
}

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

#[test]
fn test_builtin_sum_with_start() {
    let out = jit_capture(
        r#"print(sum([1, 2, 3], 10))
print(sum([], 100))
"#,
    );
    assert_output(&out, "16\n100\n");
}

#[test]
fn test_builtin_sorted_ascending() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 4, 1, 5, 9, 2, 6]))
"#,
    );
    assert_output(&out, "[1, 1, 2, 3, 4, 5, 6, 9]\n");
}

#[test]
fn test_builtin_sorted_reverse() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 4, 1, 5], reverse=True))
"#,
    );
    assert_output(&out, "[5, 4, 3, 1, 1]\n");
}

#[test]
fn test_builtin_sorted_strings() {
    let out = jit_capture(
        r#"print(sorted(["banana", "apple", "cherry"]))
"#,
    );
    assert_output(&out, "['apple', 'banana', 'cherry']\n");
}
