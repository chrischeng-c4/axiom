//! Py3.12 conformance tests for `lambda` expressions (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_funcattrs.py and
//! pattern-style tests in Lib/test/test_grammar.py): lambda as value,
//! lambda used with map/filter, lambda as sort key.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_lambda_binary_value() {
    let out = jit_capture(
        r#"add = lambda x, y: x + y
print(add(3, 4))
"#,
    );
    assert_output(&out, "7\n");
}

#[test]
fn test_lambda_with_map() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4]
print(list(map(lambda x: x * 10, nums)))
"#,
    );
    assert_output(&out, "[10, 20, 30, 40]\n");
}

#[test]
fn test_lambda_with_filter_even() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4, 5, 6]
print(list(filter(lambda x: x % 2 == 0, nums)))
"#,
    );
    assert_output(&out, "[2, 4, 6]\n");
}

#[test]
fn test_lambda_as_sort_key() {
    let out = jit_capture(
        r#"pairs = [(1, "b"), (3, "a"), (2, "c")]
pairs.sort(key=lambda p: p[1])
print(pairs)
"#,
    );
    assert_output(&out, "[(3, 'a'), (1, 'b'), (2, 'c')]\n");
}
