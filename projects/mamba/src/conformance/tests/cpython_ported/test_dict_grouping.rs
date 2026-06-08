//! Py3.12 conformance tests for dict-of-list grouping patterns
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_dict.py and
//! Lib/test/test_dictviews.py — grouping/counting idioms):
//! group-by-first-letter via `not in` + `append`, character
//! counting via `dict.get` + default, and stable ordered key
//! iteration via `sorted`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_group_words_by_first_letter() {
    let out = jit_capture(
        r#"words = ["apple", "ant", "banana", "berry", "cherry", "almond"]
groups = {}
for w in words:
    k = w[0]
    if k not in groups:
        groups[k] = []
    groups[k].append(w)
for k in sorted(groups.keys()):
    print(k, groups[k])
"#,
    );
    assert_output(
        &out,
        "a ['apple', 'ant', 'almond']\nb ['banana', 'berry']\nc ['cherry']\n",
    );
}

#[test]
fn test_char_count_via_dict_get_default() {
    let out = jit_capture(
        r#"counts = {}
for ch in "mississippi":
    counts[ch] = counts.get(ch, 0) + 1
for k in sorted(counts.keys()):
    print(k, counts[k])
"#,
    );
    assert_output(&out, "i 4\nm 1\np 2\ns 4\n");
}

#[test]
fn test_sorted_keys_after_grouping() {
    let out = jit_capture(
        r#"items = [("z", 1), ("a", 2), ("m", 3), ("a", 4)]
agg = {}
for k, v in items:
    if k not in agg:
        agg[k] = 0
    agg[k] = agg[k] + v
for k in sorted(agg.keys()):
    print(k, agg[k])
"#,
    );
    assert_output(&out, "a 6\nm 3\nz 1\n");
}
