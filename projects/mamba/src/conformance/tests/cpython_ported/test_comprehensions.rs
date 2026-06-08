//! Py3.12 conformance tests for list/dict/set comprehensions and
//! generator expressions (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_genexps.py and
//! Lib/test/test_listcomps.py):
//!   list comprehension, list comprehension with filter, dict
//!   comprehension, set comprehension, generator expression with sum.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_list_comprehension_squares() {
    let out = jit_capture(
        r#"print([x * x for x in range(5)])
"#,
    );
    assert_output(&out, "[0, 1, 4, 9, 16]\n");
}

#[test]
fn test_list_comprehension_with_filter() {
    let out = jit_capture(
        r#"print([x for x in range(10) if x % 2 == 0])
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n");
}

#[test]
fn test_dict_comprehension_squares() {
    let out = jit_capture(
        r#"print({x: x * x for x in range(4)})
"#,
    );
    assert_output(&out, "{0: 0, 1: 1, 2: 4, 3: 9}\n");
}

#[test]
fn test_set_comprehension_mod() {
    let out = jit_capture(
        r#"print({x % 3 for x in range(10)})
"#,
    );
    assert_output(&out, "{0, 1, 2}\n");
}

#[test]
fn test_generator_expression_with_sum() {
    let out = jit_capture(
        r#"print(sum(x * x for x in range(5)))
"#,
    );
    assert_output(&out, "30\n");
}
