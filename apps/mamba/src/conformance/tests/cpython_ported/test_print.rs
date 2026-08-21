//! Py3.12 conformance tests for the print() builtin (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_print.py — PrintTest
//!
//! Coverage: print() with no args, single arg of each primitive type,
//! multiple args (default space separator + trailing newline), `sep=`
//! keyword, `end=` keyword, both `sep=` and `end=` together, mixed-type
//! arg lists, and printing container literals.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_print_no_args() {
    let out = jit_capture(
        r#"print()
print()
"#,
    );
    assert_output(&out, "\n\n");
}

#[test]
fn test_print_single_str() {
    let out = jit_capture(
        r#"print("hello")
"#,
    );
    assert_output(&out, "hello\n");
}

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

#[test]
fn test_print_single_bool() {
    let out = jit_capture(
        r#"print(True)
print(False)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_print_single_none() {
    let out = jit_capture(
        r#"print(None)
"#,
    );
    assert_output(&out, "None\n");
}

#[test]
fn test_print_multiple_args_default_sep() {
    let out = jit_capture(
        r#"print(1, 2, 3)
print("a", "b", "c")
"#,
    );
    assert_output(&out, "1 2 3\na b c\n");
}

#[test]
fn test_print_sep_kwarg() {
    let out = jit_capture(
        r#"print("a", "b", "c", sep="-")
print(1, 2, 3, sep=", ")
"#,
    );
    assert_output(&out, "a-b-c\n1, 2, 3\n");
}

#[test]
fn test_print_sep_empty() {
    let out = jit_capture(
        r#"print("a", "b", "c", sep="")
"#,
    );
    assert_output(&out, "abc\n");
}

#[test]
fn test_print_end_kwarg() {
    let out = jit_capture(
        r#"print("a", end="!")
print("b")
"#,
    );
    assert_output(&out, "a!b\n");
}

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

#[test]
fn test_print_sep_and_end_together() {
    let out = jit_capture(
        r#"print("a", "b", "c", sep="-", end="!\n")
"#,
    );
    assert_output(&out, "a-b-c!\n");
}

#[test]
fn test_print_mixed_types() {
    let out = jit_capture(
        r#"print("count:", 3, "ok:", True)
"#,
    );
    assert_output(&out, "count: 3 ok: True\n");
}

#[test]
fn test_print_list_literal() {
    let out = jit_capture(
        r#"print([1, 2, 3])
print([])
"#,
    );
    assert_output(&out, "[1, 2, 3]\n[]\n");
}

#[test]
fn test_print_tuple_literal() {
    let out = jit_capture(
        r#"print((1, 2, 3))
print(())
"#,
    );
    assert_output(&out, "(1, 2, 3)\n()\n");
}

#[test]
fn test_print_dict_literal() {
    let out = jit_capture(
        r#"print({})
print({"a": 1})
"#,
    );
    assert_output(&out, "{}\n{'a': 1}\n");
}
