//! Py3.12 conformance tests for `repr` and `str` built-ins (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_repr.py — basic
//! built-in `repr` coverage):
//!   `repr` quotes strings, prints ints/lists/tuples/dicts/None/bool
//!   in canonical form; `str()` on non-string values gives the printable
//!   form (unquoted strings, canonical containers).
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_repr_quotes_string_and_canonicalizes_scalars() {
    let out = jit_capture(
        r#"print(repr("hello"))
print(repr(42))
print(repr(None))
print(repr(True))
"#,
    );
    assert_output(&out, "'hello'\n42\nNone\nTrue\n");
}

#[test]
fn test_repr_containers_match_literal_forms() {
    let out = jit_capture(
        r#"print(repr([1, 2, 3]))
print(repr((1, 2)))
print(repr({1: 2}))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n(1, 2)\n{1: 2}\n");
}

#[test]
fn test_str_of_scalars_and_containers() {
    let out = jit_capture(
        r#"print(str(42))
print(str([1, 2, 3]))
print(str(None))
"#,
    );
    assert_output(&out, "42\n[1, 2, 3]\nNone\n");
}
