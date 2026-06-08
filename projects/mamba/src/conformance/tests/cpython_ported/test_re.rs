//! Py3.12 conformance tests for the `re` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_re.py):
//!   match/search/fullmatch, group, sub, findall, split, compile flags.
//!
//! Bulk-text regex perf is excluded — it is tracked separately (#2110)
//! because per-match `MbObject::new_str()` allocations blow the memory
//! gate. The functional surface tested here uses small inputs only.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_re_match_group_one() {
    let out = jit_capture(
        r#"import re
m = re.match(r"hello (\w+)", "hello world")
print(m.group(1))
"#,
    );
    assert_output(&out, "world\n");
}

#[test]
fn test_re_search_first_hit() {
    let out = jit_capture(
        r#"import re
m = re.search(r"\d+", "abc 42 def")
print(m.group(0))
"#,
    );
    assert_output(&out, "42\n");
}

#[test]
fn test_re_sub_replace_all() {
    let out = jit_capture(
        r#"import re
print(re.sub(r"\d+", "X", "abc123def456"))
"#,
    );
    assert_output(&out, "abcXdefX\n");
}

#[test]
fn test_re_findall_numbers() {
    let out = jit_capture(
        r#"import re
print(re.findall(r"\d+", "a1 b22 c333"))
"#,
    );
    assert_output(&out, "['1', '22', '333']\n");
}

#[test]
fn test_re_split_on_commas() {
    let out = jit_capture(
        r#"import re
print(re.split(r",\s*", "a, b,c,  d"))
"#,
    );
    assert_output(&out, "['a', 'b', 'c', 'd']\n");
}

#[test]
fn test_re_fullmatch_anchored() {
    let out = jit_capture(
        r#"import re
print(re.fullmatch(r"\d+", "1234") is not None)
print(re.fullmatch(r"\d+", "12a4") is None)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}
