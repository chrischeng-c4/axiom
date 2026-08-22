//! Ported from Lib/test/test_str_ported.py
//! Integration tests: builtins/str.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_plus_concatenation() {
    let out = jit_capture(
        r#"print("ab" + "cd")
print("foo" + "" + "bar")
print("" + "x")
"#,
    );
    assert_output(&out, "abcd\nfoobar\nx\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_repetition_both_orders() {
    let out = jit_capture(
        r#"print("ab" * 3)
print(3 * "x")
print("hi" * 0)
"#,
    );
    assert_output(&out, "ababab\nxxx\n\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_in_substring_membership() {
    let out = jit_capture(
        r#"print("l" in "hello")
print("z" in "hello")
print("hel" in "hello")
print("xyz" in "hello")
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_len_on_empty_and_non_empty() {
    let out = jit_capture(
        r#"print(len("hello"))
print(len(""))
print(len("a"))
"#,
    );
    assert_output(&out, "5\n0\n1\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_default_space_padding() {
    let out = jit_capture(
        r#"print(repr("hi".ljust(5)))
print(repr("hi".rjust(5)))
print(repr("hi".center(5)))
"#,
    );
    assert_output(&out, "'hi   '\n'   hi'\n'  hi '\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_custom_fill_character() {
    let out = jit_capture(
        r#"print("hi".ljust(5, "*"))
print("hi".rjust(5, "*"))
print("hi".center(5, "*"))
"#,
    );
    assert_output(&out, "hi***\n***hi\n**hi*\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_zfill_preserves_sign() {
    let out = jit_capture(
        r#"print("42".zfill(5))
print("-7".zfill(5))
print("0".zfill(3))
"#,
    );
    assert_output(&out, "00042\n-0007\n000\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_empty_literal_len() {
    let out = jit_capture(
        r#"s = ""
print(len(s))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_literal_len() {
    let out = jit_capture(
        r#"s = "hello"
print(len(s))
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_constructor_from_int() {
    let out = jit_capture(
        r#"s = str(42)
print(s)
print(len(s))
"#,
    );
    assert_output(&out, "42\n2\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_getitem_positive() {
    let out = jit_capture(
        r#"s = "abc"
print(s[0])
print(s[1])
print(s[2])
"#,
    );
    assert_output(&out, "a\nb\nc\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_getitem_negative() {
    let out = jit_capture(
        r#"s = "abc"
print(s[-1])
print(s[-3])
"#,
    );
    assert_output(&out, "c\na\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_upper() {
    let out = jit_capture(
        r#"print("Hello".upper())
"#,
    );
    assert_output(&out, "HELLO\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_lower() {
    let out = jit_capture(
        r#"print("HeLLo".lower())
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_capitalize() {
    let out = jit_capture(
        r#"print("hello world".capitalize())
"#,
    );
    assert_output(&out, "Hello world\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_title() {
    let out = jit_capture(
        r#"print("hello world".title())
"#,
    );
    assert_output(&out, "Hello World\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_strip_whitespace() {
    let out = jit_capture(
        r#"print("  hello  ".strip())
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_lstrip_whitespace() {
    let out = jit_capture(
        r#"s = "  hello  ".lstrip()
print(s)
print(len(s))
"#,
    );
    assert_output(&out, "hello  \n7\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_rstrip_whitespace() {
    let out = jit_capture(
        r#"s = "  hello  ".rstrip()
print(s)
print(len(s))
"#,
    );
    assert_output(&out, "  hello\n7\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_startswith_true() {
    let out = jit_capture(
        r#"print("hello world".startswith("hello"))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_startswith_false() {
    let out = jit_capture(
        r#"print("hello world".startswith("world"))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_endswith_true() {
    let out = jit_capture(
        r#"print("hello world".endswith("world"))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_endswith_false() {
    let out = jit_capture(
        r#"print("hello world".endswith("hello"))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_find_present() {
    let out = jit_capture(
        r#"print("hello world".find("world"))
"#,
    );
    assert_output(&out, "6\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_find_absent_returns_minus_one() {
    let out = jit_capture(
        r#"print("hello".find("xyz"))
"#,
    );
    assert_output(&out, "-1\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_index_present() {
    let out = jit_capture(
        r#"print("hello world".index("world"))
"#,
    );
    assert_output(&out, "6\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_count_substr() {
    let out = jit_capture(
        r#"print("banana".count("a"))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_count_absent_is_zero() {
    let out = jit_capture(
        r#"print("banana".count("z"))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_replace_basic() {
    let out = jit_capture(
        r#"print("hello world".replace("world", "python"))
"#,
    );
    assert_output(&out, "hello python\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_replace_all_occurrences() {
    let out = jit_capture(
        r#"print("aaa".replace("a", "b"))
"#,
    );
    assert_output(&out, "bbb\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_replace_absent_is_noop() {
    let out = jit_capture(
        r#"print("hello".replace("z", "x"))
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_split_default_whitespace() {
    let out = jit_capture(
        r#"parts = "a b c".split()
print(len(parts))
print(parts[0])
print(parts[2])
"#,
    );
    assert_output(&out, "3\na\nc\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_split_explicit_separator() {
    let out = jit_capture(
        r#"parts = "a,b,c".split(",")
print(len(parts))
print(parts[1])
"#,
    );
    assert_output(&out, "3\nb\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_join_with_list() {
    let out = jit_capture(
        r#"print(",".join(["a", "b", "c"]))
"#,
    );
    assert_output(&out, "a,b,c\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_join_empty_iterable() {
    let out = jit_capture(
        r#"print(",".join([]))
"#,
    );
    assert_output(&out, "\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isdigit_true() {
    let out = jit_capture(
        r#"print("123".isdigit())
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isdigit_false() {
    let out = jit_capture(
        r#"print("12a".isdigit())
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isalpha_true() {
    let out = jit_capture(
        r#"print("abc".isalpha())
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isalpha_false_with_digit() {
    let out = jit_capture(
        r#"print("abc1".isalpha())
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_concatenation_operator() {
    let out = jit_capture(
        r#"print("hello" + " " + "world")
"#,
    );
    assert_output(&out, "hello world\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_repetition_operator() {
    let out = jit_capture(
        r#"print("ab" * 3)
"#,
    );
    assert_output(&out, "ababab\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_equal_same_content() {
    let out = jit_capture(
        r#"print("hello" == "hello")
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_not_equal_different_content() {
    let out = jit_capture(
        r#"print("hello" == "world")
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_bool_constructor_from_str_empty() {
    let out = jit_capture(
        r#"print(bool(""))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool("x"))
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_contains_present() {
    let out = jit_capture(
        r#"print("ell" in "hello")
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_contains_absent() {
    let out = jit_capture(
        r#"print("xyz" in "hello")
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_bool_constructor_from_str_nonempty() {
    let out = jit_capture(
        r#"print(bool("x"))
print(bool("0"))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_bool_str_conversion() {
    let out = jit_capture(
        r#"print(str(True))
print(str(False))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_string_ascii_lowercase() {
    let out = jit_capture(
        r#"import string
print(string.ascii_lowercase)
"#,
    );
    assert_output(&out, "abcdefghijklmnopqrstuvwxyz\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_string_ascii_uppercase() {
    let out = jit_capture(
        r#"import string
print(string.ascii_uppercase)
"#,
    );
    assert_output(&out, "ABCDEFGHIJKLMNOPQRSTUVWXYZ\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_string_digits() {
    let out = jit_capture(
        r#"import string
print(string.digits)
"#,
    );
    assert_output(&out, "0123456789\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_string_hexdigits() {
    let out = jit_capture(
        r#"import string
print(string.hexdigits)
"#,
    );
    assert_output(&out, "0123456789abcdefABCDEF\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_string_octdigits() {
    let out = jit_capture(
        r#"import string
print(string.octdigits)
"#,
    );
    assert_output(&out, "01234567\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_string_ascii_letters_concat() {
    let out = jit_capture(
        r#"import string
print(len(string.ascii_letters))
print(string.ascii_letters[:5])
print(string.ascii_letters[-5:])
"#,
    );
    assert_output(&out, "52\nabcde\nVWXYZ\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_basic_and_reverse() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[:])
print(s[2:])
print(s[:3])
print(s[2:5])
print(s[::-1])
"#,
    );
    assert_output(&out, "abcdefgh\ncdefgh\nabc\ncde\nhgfedcba\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_step_and_negative() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(s[::2])
print(s[1::2])
print(s[-3:])
print(s[:-3])
print(s[-5:-2])
"#,
    );
    assert_output(&out, "aceg\nbdfh\nfgh\nabcde\ndef\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_out_of_range_and_len() {
    let out = jit_capture(
        r#"s = "abcdefgh"
print(repr(s[10:]))
print(repr(s[5:2]))
print(len(s[2:5]))
"#,
    );
    assert_output(&out, "''\n''\n3\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_index_returns_first_occurrence() {
    let out = jit_capture(
        r#"print("hello".index("l"))
print("banana".index("a"))
print("hello".index("o"))
"#,
    );
    assert_output(&out, "2\n1\n4\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_index_raises_on_miss() {
    let out = jit_capture(
        r#"try:
    print("hello".index("z"))
except ValueError:
    print("not found")
try:
    print("abc".index("d"))
except ValueError:
    print("absent")
"#,
    );
    assert_output(&out, "not found\nabsent\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_rfind_last_occurrence_and_miss() {
    let out = jit_capture(
        r#"print("hello".rfind("l"))
print("banana".rfind("a"))
print("hello".rfind("z"))
"#,
    );
    assert_output(&out, "3\n5\n-1\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_inf() {
    let out = jit_capture(r#"print(float("inf"))"#);
    assert_output(&out, "inf\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_negative_inf() {
    let out = jit_capture(r#"print(float("-inf"))"#);
    assert_output(&out, "-inf\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_nan() {
    let out = jit_capture(r#"print(float("nan"))"#);
    assert_output(&out, "nan\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_infinity_long() {
    let out = jit_capture(r#"print(float("Infinity"))"#);
    assert_output(&out, "inf\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_scientific_positive() {
    let out = jit_capture(r#"print(float("1e10"))"#);
    assert_output(&out, "10000000000.0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_scientific_negative_exp() {
    let out = jit_capture(r#"print(float("1.5e-3"))"#);
    assert_output(&out, "0.0015\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_negative_zero() {
    let out = jit_capture(r#"print(str(-0.0))"#);
    assert_output(&out, "-0.0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_replace_substitutes_substring() {
    let out = jit_capture(
        r#"print("hello world".replace("world", "Python"))
print("aaa".replace("a", "b"))
print("xxx".replace("y", "z"))
"#,
    );
    assert_output(&out, "hello Python\nbbb\nxxx\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_split_and_join_roundtrip() {
    let out = jit_capture(
        r#"parts = "a,b,c".split(",")
print(parts)
print("-".join(parts))
print(",".join(["x", "y", "z"]))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\na-b-c\nx,y,z\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_find_and_count() {
    let out = jit_capture(
        r#"print("hello".find("l"))
print("hello".find("z"))
print("banana".count("a"))
print("banana".count("na"))
"#,
    );
    assert_output(&out, "2\n-1\n3\n2\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_startswith_and_endswith() {
    let out = jit_capture(
        r#"print("hello".startswith("he"))
print("hello".startswith("lo"))
print("hello".endswith("lo"))
print("hello".endswith("he"))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_repr_and_str_on_ints() {
    let out = jit_capture(
        r#"print(repr(42))
print(repr(-7))
print(str(1000000))
print(str(0))
"#,
    );
    assert_output(&out, "42\n-7\n1000000\n0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_no_args() {
    let out = jit_capture(
        r#"print()
print()
"#,
    );
    assert_output(&out, "\n\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_single_str() {
    let out = jit_capture(
        r#"print("hello")
"#,
    );
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_single_int() {
    let out = jit_capture(
        r#"print(42)
print(-7)
print(0)
"#,
    );
    assert_output(&out, "42\n-7\n0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_single_float() {
    let out = jit_capture(
        r#"print(3.14)
print(-2.5)
print(0.0)
"#,
    );
    assert_output(&out, "3.14\n-2.5\n0.0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_single_bool() {
    let out = jit_capture(
        r#"print(True)
print(False)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_single_none() {
    let out = jit_capture(
        r#"print(None)
"#,
    );
    assert_output(&out, "None\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_multiple_args_default_sep() {
    let out = jit_capture(
        r#"print(1, 2, 3)
print("a", "b", "c")
"#,
    );
    assert_output(&out, "1 2 3\na b c\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_custom_separator() {
    let out = jit_capture(
        r#"print("a", "b", "c", sep="-")
print(1, 2, 3, sep=", ")
"#,
    );
    assert_output(&out, "a-b-c\n1, 2, 3\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_sep_empty() {
    let out = jit_capture(
        r#"print("a", "b", "c", sep="")
"#,
    );
    assert_output(&out, "abc\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_end_kwarg() {
    let out = jit_capture(
        r#"print("a", end="!")
print("b")
"#,
    );
    assert_output(&out, "a!b\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_end_empty() {
    let out = jit_capture(
        r#"print("a", end="")
print("b", end="")
print("c")
"#,
    );
    assert_output(&out, "abc\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_sep_and_end_together() {
    let out = jit_capture(
        r#"print("a", "b", "c", sep="-", end="!\n")
"#,
    );
    assert_output(&out, "a-b-c!\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_mixed_types() {
    let out = jit_capture(
        r#"print("count:", 3, "ok:", True)
"#,
    );
    assert_output(&out, "count: 3 ok: True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_list_literal() {
    let out = jit_capture(
        r#"print([1, 2, 3])
print([])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[]\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_tuple_literal() {
    let out = jit_capture(
        r#"print((1, 2, 3))
print(())
"#,
    );
    assert_output(&out, "(1, 2, 3)\n()\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_dict_literal() {
    let out = jit_capture(
        r#"print({})
print({"a": 1})
"#,
    );
    assert_output(&out, "{}\n{'a': 1}\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_empty_separator_concatenates() {
    let out = jit_capture(
        r#"print("x", "y", "z", sep="")
"#,
    );
    assert_output(&out, "xyz\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_custom_end_suppresses_newline() {
    let out = jit_capture(
        r#"print("hello", end="")
print(" world")
print("a", end="|")
print("b", end="|")
print("c")
"#,
    );
    assert_output(&out, "hello world\na|b|c\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_default_whitespace_split() {
    let out = jit_capture(
        r#"print("a b  c".split())
print("  hi  ".split())
print("one two three".split())
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n['hi']\n['one', 'two', 'three']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_split_and_rsplit_with_maxsplit() {
    let out = jit_capture(
        r#"print("a,b,c,d".split(",", 2))
print("a,b,c,d".rsplit(",", 2))
"#,
    );
    assert_output(&out, "['a', 'b', 'c,d']\n['a,b', 'c', 'd']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_splitlines_and_consecutive_delim() {
    let out = jit_capture(
        r#"print("a\nb\nc".splitlines())
print("hello".split("l"))
print("a,,b".split(","))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n['he', '', 'o']\n['a', '', 'b']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_partition_splits_on_first_delim() {
    let out = jit_capture(
        r#"print("a,b,c".partition(","))
"#,
    );
    assert_output(&out, "('a', ',', 'b,c')\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_rpartition_splits_on_last_delim() {
    let out = jit_capture(
        r#"print("a,b,c".rpartition(","))
"#,
    );
    assert_output(&out, "('a,b', ',', 'c')\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_partition_no_delim_returns_original_plus_empties() {
    let out = jit_capture(
        r#"print("nodelim".partition(","))
"#,
    );
    assert_output(&out, "('nodelim', '', '')\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_iter_str_for_loop() {
    let out = jit_capture(
        r#"for c in "abc":
    print(c)
"#,
    );
    assert_output(&out, "a\nb\nc\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_int_conversion_from_str_and_float() {
    let out = jit_capture(
        r#"print(int("42"))
print(int(3.7))
print(int("-10"))
"#,
    );
    assert_output(&out, "42\n3\n-10\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_conversion_from_str_and_int() {
    let out = jit_capture(
        r#"print(float("3.14"))
print(float(2))
print(float("-0.5"))
"#,
    );
    assert_output(&out, "3.14\n2.0\n-0.5\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_constructor_on_numbers() {
    let out = jit_capture(
        r#"print(str(42))
print(str(3.14))
print(str(-7))
"#,
    );
    assert_output(&out, "42\n3.14\n-7\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_variable_interpolation() {
    let out = jit_capture(
        r#"x = 42
print(f"x={x}")
"#,
    );
    assert_output(&out, "x=42\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_expression_interpolation() {
    let out = jit_capture(
        r#"print(f"{2+3}")
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_right_align_width() {
    let out = jit_capture(
        r#"print(f"{'hi':>5}")
"#,
    );
    assert_output(&out, "   hi\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_float_precision() {
    let out = jit_capture(
        r#"print(f"{3.14:.2f}")
"#,
    );
    assert_output(&out, "3.14\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_alt_hex_format() {
    let out = jit_capture(
        r#"print(f"{255:#x}")
"#,
    );
    assert_output(&out, "0xff\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_percentage_format() {
    let out = jit_capture(
        r#"print(f"{0.5:.0%}")
"#,
    );
    assert_output(&out, "50%\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_upper_lower_swapcase() {
    let out = jit_capture(
        r#"print("Hello".upper())
print("Hello".lower())
print("Hello".swapcase())
"#,
    );
    assert_output(&out, "HELLO\nhello\nhELLO\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_strip_default_whitespace() {
    let out = jit_capture(
        r#"print(repr("  hi  ".strip()))
print(repr("  hi  ".lstrip()))
print(repr("  hi  ".rstrip()))
"#,
    );
    assert_output(&out, "'hi'\n'hi  '\n'  hi'\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_strip_with_custom_chars() {
    let out = jit_capture(
        r#"print("xxhixx".strip("x"))
print("---note---".strip("-"))
print("abcXYZcba".strip("abc"))
"#,
    );
    assert_output(&out, "hi\nnote\nXYZ\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_for_loop_iterates_characters() {
    let out = jit_capture(
        r#"out = []
for c in "abc":
    out.append(c)
print(out)
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_list_comprehension_over_string() {
    let out = jit_capture(
        r#"print([c for c in "hello"])
print([c.upper() for c in "abc"])
"#,
    );
    assert_output(&out, "['h', 'e', 'l', 'l', 'o']\n['A', 'B', 'C']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_list_constructor_on_string() {
    let out = jit_capture(
        r#"print(list("hi"))
print(list(""))
print(list("a"))
"#,
    );
    assert_output(&out, "['h', 'i']\n[]\n['a']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_center_with_fill() {
    let out = jit_capture(r#"print("abc".center(7, "*"))"#);
    assert_output(&out, "**abc**\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_ljust_with_fill() {
    let out = jit_capture(r#"print("abc".ljust(7, "-"))"#);
    assert_output(&out, "abc----\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_rjust_with_fill() {
    let out = jit_capture(r#"print("abc".rjust(7, "-"))"#);
    assert_output(&out, "----abc\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_zfill() {
    let out = jit_capture(r#"print("42".zfill(5))"#);
    assert_output(&out, "00042\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_rfind_present() {
    let out = jit_capture(r#"print("hello".rfind("l"))"#);
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_rfind_absent() {
    let out = jit_capture(r#"print("hello".rfind("z"))"#);
    assert_output(&out, "-1\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_strip_chars() {
    let out = jit_capture(r#"print("xxhelloxx".strip("x"))"#);
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_lstrip_chars() {
    let out = jit_capture(r#"print("xxhello".lstrip("x"))"#);
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_rstrip_chars() {
    let out = jit_capture(r#"print("helloxx".rstrip("x"))"#);
    assert_output(&out, "hello\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isalnum_true() {
    let out = jit_capture(r#"print("abc123".isalnum())"#);
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isalnum_false_with_space() {
    let out = jit_capture(r#"print("abc 123".isalnum())"#);
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isspace_true() {
    let out = jit_capture(r#"print("   ".isspace())"#);
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isspace_false() {
    let out = jit_capture(r#"print("a b".isspace())"#);
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isupper_true() {
    let out = jit_capture(r#"print("ABC".isupper())"#);
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isupper_false() {
    let out = jit_capture(r#"print("Abc".isupper())"#);
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_islower_true() {
    let out = jit_capture(r#"print("abc".islower())"#);
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_islower_false() {
    let out = jit_capture(r#"print("Abc".islower())"#);
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_istitle_true() {
    let out = jit_capture(r#"print("Title Case".istitle())"#);
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_istitle_false() {
    let out = jit_capture(r#"print("not title".istitle())"#);
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_range() {
    let out = jit_capture(r#"print("hello"[1:4])"#);
    assert_output(&out, "ell\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_reverse() {
    let out = jit_capture(r#"print("hello"[::-1])"#);
    assert_output(&out, "olleh\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_step() {
    let out = jit_capture(r#"print("abcdef"[::2])"#);
    assert_output(&out, "ace\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_to_end() {
    let out = jit_capture(r#"print("hello"[2:])"#);
    assert_output(&out, "llo\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_slice_from_start() {
    let out = jit_capture(r#"print("hello"[:3])"#);
    assert_output(&out, "hel\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_repr() {
    let out = jit_capture(r#"print(repr("abc"))"#);
    assert_output(&out, "'abc'\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_encode_utf8() {
    let out = jit_capture(r#"print("hello".encode("utf-8"))"#);
    assert_output(&out, "b'hello'\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_replace_with_count() {
    let out = jit_capture(r#"print("aaaa".replace("a", "b", 2))"#);
    assert_output(&out, "bbaa\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_split_max_splits() {
    let out = jit_capture(r#"print("a,b,c,d".split(",", 2))"#);
    assert_output(&out, "['a', 'b', 'c,d']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_rsplit_max_splits() {
    let out = jit_capture(r#"print("a,b,c,d".rsplit(",", 2))"#);
    assert_output(&out, "['a,b', 'c', 'd']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_splitlines() {
    let out = jit_capture(r#"print("a\nb\nc".splitlines())"#);
    assert_output(&out, "['a', 'b', 'c']\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_int_float_str_conversion() {
    let out = jit_capture(
        r#"print(int(3.14))
print(int(-2.7))
print(float(5))
print(float("3.14"))
print(str(2.5))
print(str(0.0))
"#,
    );
    assert_output(&out, "3\n-2\n5.0\n3.14\n2.5\n0.0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_capitalize_and_title() {
    let out = jit_capture(
        r#"print("hello world".capitalize())
print("hello world".title())
print("PYTHON".capitalize())
"#,
    );
    assert_output(&out, "Hello world\nHello World\nPython\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_expandtabs_and_center_fill() {
    let out = jit_capture(
        r#"print("ab\tcd".expandtabs(4))
print("hello".center(11, "*"))
print("x".center(5, "-"))
"#,
    );
    assert_output(&out, "ab  cd\n***hello***\n--x--\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_predicate_combo() {
    let out = jit_capture(
        r#"print("abc".isalpha())
print("123".isdigit())
print("abc123".isalnum())
print("abc!".isalnum())
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_arithmetic_and_method_call() {
    let out = jit_capture(
        r#"name = "Alice"
age = 30
print(f"{age * 2}")
print(f"{1 + 2 + 3}")
print(f"len={len(name)}")
print(f"{name.upper()}")
"#,
    );
    assert_output(&out, "60\n6\nlen=5\nALICE\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_repr_conversion() {
    let out = jit_capture(
        r#"name = "Alice"
print(f"{name!r}")
print(f"{'hi'!r}")
print(f"{42!r}")
"#,
    );
    assert_output(&out, "'Alice'\n'hi'\n42\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_ternary_expression() {
    let out = jit_capture(
        r#"age = 30
print(f"{'yes' if age > 18 else 'no'}")
print(f"{'adult' if age >= 18 else 'child'}")
print(f"{'+' if 1 > 0 else '-'}")
"#,
    );
    assert_output(&out, "yes\nadult\n+\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_float_str_conversion_simple() {
    let out = jit_capture(
        r#"print(str(1.5))
print(str(-2.0))
"#,
    );
    assert_output(&out, "1.5\n-2.0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_prints_non_ascii_characters() {
    let out = jit_capture(
        r#"print("é")
print("ñ")
print("ü")
"#,
    );
    assert_output(&out, "é\nñ\nü\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_ord_chr_on_non_ascii_codepoints() {
    let out = jit_capture(
        r#"print(ord("é"))
print(chr(233))
print(ord(chr(255)))
"#,
    );
    assert_output(&out, "233\né\n255\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_len_counts_characters_not_bytes() {
    let out = jit_capture(
        r#"print(len("café"))
print(len("año"))
print(len("ascii"))
"#,
    );
    assert_output(&out, "4\n3\n5\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_indexing_first_and_last() {
    let out = jit_capture(
        r#"print("hello"[0])
print("hello"[-1])
print("hello"[2])
"#,
    );
    assert_output(&out, "h\no\nl\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_concat_len_and_repeat_chain() {
    let out = jit_capture(
        r#"a = "hello"
b = "world"
print(a + " " + b)
print(len(a + b))
print((a + b) * 2)
"#,
    );
    assert_output(&out, "hello world\n10\nhelloworldhelloworld\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_join_with_identifier_separator() {
    let out = jit_capture(
        r#"a = "hello"
b = "world"
sep = "-"
print(sep.join([a, b]))
print(("=" * 10))
"#,
    );
    assert_output(&out, "hello-world\n==========\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_repeat_of_concat_and_zero() {
    let out = jit_capture(
        r#"s = "ab"
print(s * 0)
print((s + "c") * 3)
print("x" * 5 + "y")
"#,
    );
    assert_output(&out, "\nabcabcabc\nxxxxxy\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_int_str_conversion() {
    let out = jit_capture(
        r#"print(str(42))
print(str(-7))
print(str(0))
"#,
    );
    assert_output(&out, "42\n-7\n0\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_none_true_false_literals() {
    let out = jit_capture(
        r#"print(None)
print(True)
print(False)
"#,
    );
    assert_output(&out, "None\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_basic_containers() {
    let out = jit_capture(
        r#"print([1, 2, 3])
print((1, 2, 3))
print({1, 2, 3})
print({"a": 1})
"#,
    );
    assert_output(&out, "[1, 2, 3]\n(1, 2, 3)\n{1, 2, 3}\n{'a': 1}\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_print_containers_holding_specials() {
    let out = jit_capture(
        r#"print([None, True, False])
print((None,))
print([True, False])
"#,
    );
    assert_output(&out, "[None, True, False]\n(None,)\n[True, False]\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_float_precision_2() {
    let out = jit_capture(
        r#"pi = 3.14159
print(f"pi ~ {pi:.2f}")
print(f"pi to 4 = {pi:.4f}")
"#,
    );
    assert_output(&out, "pi ~ 3.14\npi to 4 = 3.1416\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_int_width_and_base() {
    let out = jit_capture(
        r#"n = 42
print(f"padded: {n:5}")
print(f"hex: {n:x}")
print(f"bin: {n:b}")
"#,
    );
    assert_output(&out, "padded:    42\nhex: 2a\nbin: 101010\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_fstring_embedded_expressions_and_methods() {
    let out = jit_capture(
        r#"name = "world"
n = 42
print(f"hello {name}")
print(f"{n} squared is {n * n}")
print(f"upper: {name.upper()}")
print(f"len = {len(name)}")
print(f"{'x' * 3}")
"#,
    );
    assert_output(
        &out,
        "hello world\n42 squared is 1764\nupper: WORLD\nlen = 5\nxxx\n",
    );
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isalpha_isdigit_isalnum() {
    let out = jit_capture(
        r#"print("abc".isalpha())
print("HELLO".isalpha())
print("123".isdigit())
print("a1".isalnum())
print("a 1".isalnum())
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_isspace() {
    let out = jit_capture(
        r#"print("  ".isspace())
print(" \t\n".isspace())
print(" a ".isspace())
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_islower_isupper() {
    let out = jit_capture(
        r#"print("hello".islower())
print("ABC".islower())
print("ABC".isupper())
print("Abc".isupper())
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nFalse\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_str_title_transform() {
    let out = jit_capture(
        r#"print("hello world".title())
print("HELLO WORLD".title())
"#,
    );
    assert_output(&out, "Hello World\nHello World\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_struct_pack_unpack_splat_roundtrip() {
    // Variadic splat: struct.pack(fmt, *args) must deliver all unpacked
    // args to the native shim. Original #2098 symptom was `sum(0..63)`
    // collapsing to 0 — args silently truncated by call-site reshape.
    let out = jit_capture(
        r#"import struct
values = tuple(range(8))
packed = struct.pack("8B", *values)
unpacked = struct.unpack("8B", packed)
print(unpacked == values)
print(sum(unpacked))
"#,
    );
    assert_output(&out, "True\n28\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_struct_calcsize_assert_raises_variadic_shape() {
    // The exact 3-arg form from the #2098 symptom:
    // `assertRaises(exc_type, callable, arg)`. Originally tripped the
    // Cranelift verifier; must now compile and execute cleanly.
    let out = jit_capture(
        r#"import unittest
import struct

class T(unittest.TestCase):
    def test_calcsize_z_format(self):
        self.assertRaises(struct.error, struct.calcsize, "Z")

t = T("test_calcsize_z_format")
t.test_calcsize_z_format()
print("ok")
"#,
    );
    assert_output(&out, "ok\n");
}

/// Ported from `Lib/test/test_str_ported.py`.
#[test]
fn test_struct_pack_variadic_large_arity() {
    // Stress: 16-arg variadic splat. Larger than typical 3-4 arg
    // assertRaises shapes — catches arity-handling regressions where
    // a small fixed limit might paper over the smaller case.
    let out = jit_capture(
        r#"import struct
values = tuple(range(16))
packed = struct.pack("16B", *values)
unpacked = struct.unpack("16B", packed)
print(unpacked == values)
"#,
    );
    assert_output(&out, "True\n");
}

