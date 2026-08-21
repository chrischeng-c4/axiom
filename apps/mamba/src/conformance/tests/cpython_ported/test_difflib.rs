//! Py3.12 conformance tests for the `difflib` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_difflib.py):
//!   get_close_matches.
//!
//! `difflib.ndiff` and `SequenceMatcher` are intentionally excluded —
//! both currently surface as `AttributeError` under mamba; deferred as
//! separate runtime gaps.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_difflib_get_close_matches_typo() {
    let out = jit_capture(
        r#"import difflib
print(difflib.get_close_matches("appel", ["apple", "ape", "banana"]))
"#,
    );
    assert_output(&out, "['apple', 'ape']\n");
}

#[test]
fn test_difflib_get_close_matches_no_hit() {
    let out = jit_capture(
        r#"import difflib
print(difflib.get_close_matches("xyz", ["apple", "banana", "cherry"]))
"#,
    );
    assert_output(&out, "[]\n");
}

#[test]
fn test_difflib_get_close_matches_exact() {
    let out = jit_capture(
        r#"import difflib
print(difflib.get_close_matches("apple", ["apple", "ape", "banana"]))
"#,
    );
    assert_output(&out, "['apple', 'ape']\n");
}
