//! Py3.12 conformance tests for dict-as-counter idioms (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py — counter
//! pattern sections): character histogram via `in`-guarded write and
//! word histogram via `dict.get` with a default.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_char_histogram_via_in_guard() {
    let out = jit_capture(
        r#"d = {}
for c in "hello world":
    if c in d:
        d[c] = d[c] + 1
    else:
        d[c] = 1
print(sorted(d.items()))
"#,
    );
    assert_output(
        &out,
        "[(' ', 1), ('d', 1), ('e', 1), ('h', 1), ('l', 3), ('o', 2), ('r', 1), ('w', 1)]\n",
    );
}

#[test]
fn test_word_histogram_via_get_default() {
    let out = jit_capture(
        r#"words = ["apple", "bee", "cat", "apple", "bee", "apple"]
counts = {}
for w in words:
    counts[w] = counts.get(w, 0) + 1
print(sorted(counts.items()))
"#,
    );
    assert_output(&out, "[('apple', 3), ('bee', 2), ('cat', 1)]\n");
}

#[test]
fn test_counter_largest_via_max_over_keys() {
    let out = jit_capture(
        r#"counts = {"a": 3, "b": 1, "c": 5, "d": 2}
items = sorted(counts.items(), key=lambda kv: -kv[1])
print(items[0])
print(items[-1])
"#,
    );
    assert_output(&out, "('c', 5)\n('b', 1)\n");
}
