//! Py3.12 conformance tests for the `fnmatch` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_fnmatch.py):
//!   fnmatch, fnmatchcase, filter.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_fnmatch_star_extension() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.fnmatch("foo.py", "*.py"))
print(fnmatch.fnmatch("foo.txt", "*.py"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_fnmatch_question_mark() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.fnmatch("ab", "?b"))
print(fnmatch.fnmatch("abc", "?b"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_fnmatch_filter_only_py() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.filter(["a.py", "b.txt", "c.py", "d.md"], "*.py"))
"#,
    );
    assert_output(&out, "['a.py', 'c.py']\n");
}

#[test]
fn test_fnmatch_char_class() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.fnmatch("a1", "a[0-9]"))
print(fnmatch.fnmatch("aa", "a[0-9]"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}
