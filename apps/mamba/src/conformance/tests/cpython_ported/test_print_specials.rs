//! Py3.12 conformance tests for printing special values (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_print.py — non-string
//! sections):
//!   `None`/`True`/`False` literals, list/tuple/set/dict containers,
//!   and containers holding the literals (None inside list/tuple).
//!
//! @issue #759

use super::{assert_output, jit_capture};

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
