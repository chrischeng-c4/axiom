use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/keyword/iskeyword_case_sensitive.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_iskeyword_case_sensitive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "iskeyword_case_sensitive"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.iskeyword: iskeyword is case-sensitive: 'class'/'True' are keywords, 'Class'/'CLASS'/'true'/'FALSE' are not"""
import keyword

for word, expected in [
    ("class", True), ("Class", False), ("CLASS", False),
    ("True", True), ("true", False), ("FALSE", False),
]:
    assert keyword.iskeyword(word) == expected, (word, expected)

print("iskeyword_case_sensitive OK")
"###);
    assert_output(&out, r###"iskeyword_case_sensitive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/iskeyword_hard_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_iskeyword_hard_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "iskeyword_hard_keywords"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.iskeyword: iskeyword returns True for every hard keyword in kwlist (all 35)"""
import keyword

hard = [
    "False", "None", "True", "and", "as", "assert", "async", "await",
    "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
]
for kw in hard:
    assert keyword.iskeyword(kw), f"{kw!r} should be a keyword"

print("iskeyword_hard_keywords OK")
"###);
    assert_output(&out, r###"iskeyword_hard_keywords OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/iskeyword_rejects_non_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_iskeyword_rejects_non_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "iskeyword_rejects_non_keywords"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.iskeyword: iskeyword returns False for ordinary identifiers, empty string, and soft keywords"""
import keyword

non_keywords = ["user", "data", "value", "hello", "match_", "", "match", "case", "type"]
for word in non_keywords:
    assert not keyword.iskeyword(word), f"{word!r} should not be a keyword"

print("iskeyword_rejects_non_keywords OK")
"###);
    assert_output(&out, r###"iskeyword_rejects_non_keywords OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/issoftkeyword_soft_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_issoftkeyword_soft_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "issoftkeyword_soft_keywords"
# subject = "keyword.issoftkeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.issoftkeyword: issoftkeyword is True for soft keywords (match/case/type/_) and False for hard keywords"""
import keyword

for soft in ["match", "case", "type", "_"]:
    assert keyword.issoftkeyword(soft), f"{soft!r} should be a soft keyword"
for hard in ["class", "def", "return", "if", "import"]:
    assert not keyword.issoftkeyword(hard), f"{hard!r} is hard, not soft"
assert not keyword.issoftkeyword("xyzzy"), "ordinary name is not a soft keyword"

print("issoftkeyword_soft_keywords OK")
"###);
    assert_output(&out, r###"issoftkeyword_soft_keywords OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/kwlist_has_35_hard_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_kwlist_has_35_hard_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "kwlist_has_35_hard_keywords"
# subject = "keyword.kwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.kwlist: kwlist contains exactly 35 hard keywords on CPython 3.12, including async/await (PEP 492)"""
import keyword

assert len(keyword.kwlist) == 35, f"expected 35 hard keywords, got {len(keyword.kwlist)}"
assert "async" in keyword.kwlist, "async (PEP 492) must be a hard keyword"
assert "await" in keyword.kwlist, "await (PEP 492) must be a hard keyword"

print("kwlist_has_35_hard_keywords OK")
"###);
    assert_output(&out, r###"kwlist_has_35_hard_keywords OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/kwlist_matches_iskeyword.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_kwlist_matches_iskeyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "kwlist_matches_iskeyword"
# subject = "keyword.kwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.kwlist: kwlist is sorted, duplicate-free, and iskeyword(x) == (x in kwlist) for every tested value"""
import keyword

kwlist = keyword.kwlist
assert kwlist == sorted(kwlist), "kwlist must be sorted"
assert len(kwlist) == len(set(kwlist)), "kwlist must have no duplicates"

# iskeyword(x) agrees with membership in kwlist for hard keywords, ordinary
# names, and soft keywords alike.
for word in kwlist + ["user", "data", "match", "case", "type", "CLASS"]:
    assert keyword.iskeyword(word) == (word in kwlist), (
        f"iskeyword({word!r}) inconsistent with kwlist membership"
    )

print("kwlist_matches_iskeyword OK")
"###);
    assert_output(&out, r###"kwlist_matches_iskeyword OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/soft_and_hard_lists_disjoint.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_soft_and_hard_lists_disjoint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "soft_and_hard_lists_disjoint"
# subject = "keyword.softkwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.softkwlist: kwlist and softkwlist are disjoint; soft keywords are absent from kwlist and bind as ordinary names via exec"""
import keyword

# The two lists never overlap.
for kw in keyword.kwlist:
    assert kw not in keyword.softkwlist, f"{kw!r} in both lists"
for sk in keyword.softkwlist:
    assert sk not in keyword.kwlist, f"{sk!r} in both lists"

# Every soft keyword binds as an ordinary identifier at runtime via exec.
ns = {}
for sk in keyword.softkwlist:
    exec(f"{sk} = 42", ns)
    assert ns[sk] == 42, f"soft keyword {sk!r} should bind to 42"

print("soft_and_hard_lists_disjoint OK")
"###);
    assert_output(&out, r###"soft_and_hard_lists_disjoint OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/softkwlist_exact_contents.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_softkwlist_exact_contents() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "softkwlist_exact_contents"
# subject = "keyword.softkwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.softkwlist: softkwlist is exactly ['_', 'case', 'match', 'type'] (sorted) on CPython 3.12 (PEP 695 'type')"""
import keyword

assert keyword.softkwlist == ["_", "case", "match", "type"], (
    f"unexpected softkwlist={keyword.softkwlist!r}"
)
assert keyword.softkwlist == sorted(keyword.softkwlist), "softkwlist must be sorted"

print("softkwlist_exact_contents OK")
"###);
    assert_output(&out, r###"softkwlist_exact_contents OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_all_soft_keywords_can_be_used_as_names.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_all_soft_keywords_can_be_used_as_names() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_all_soft_keywords_can_be_used_as_names"
# subject = "cpython.test_keyword.Test_iskeyword.test_all_soft_keywords_can_be_used_as_names"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_all_soft_keywords_can_be_used_as_names
"""Auto-ported test: Test_iskeyword::test_all_soft_keywords_can_be_used_as_names (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---
for key in keyword.softkwlist:
    exec(f'{key} = 42')
print("Test_iskeyword::test_all_soft_keywords_can_be_used_as_names: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_all_soft_keywords_can_be_used_as_names: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_async_and_await_are_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_async_and_await_are_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_async_and_await_are_keywords"
# subject = "cpython.test_keyword.Test_iskeyword.test_async_and_await_are_keywords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_async_and_await_are_keywords
"""Auto-ported test: Test_iskeyword::test_async_and_await_are_keywords (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---

assert 'async' in keyword.kwlist

assert 'await' in keyword.kwlist
print("Test_iskeyword::test_async_and_await_are_keywords: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_async_and_await_are_keywords: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_changing_the_kwlist_does_not_affect_iskeyword.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_changing_the_kwlist_does_not_affect_iskeyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_changing_the_kwlist_does_not_affect_iskeyword"
# subject = "cpython.test_keyword.Test_iskeyword.test_changing_the_kwlist_does_not_affect_iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_changing_the_kwlist_does_not_affect_iskeyword
"""Auto-ported test: Test_iskeyword::test_changing_the_kwlist_does_not_affect_iskeyword (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---
oldlist = keyword.kwlist
pass
keyword.kwlist = ['its', 'all', 'eggs', 'beans', 'and', 'a', 'slice']

assert not keyword.iskeyword('eggs')
print("Test_iskeyword::test_changing_the_kwlist_does_not_affect_iskeyword: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_changing_the_kwlist_does_not_affect_iskeyword: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_changing_the_softkwlist_does_not_affect_issoftkeyword.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_changing_the_softkwlist_does_not_affect_issoftkeyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_changing_the_softkwlist_does_not_affect_issoftkeyword"
# subject = "cpython.test_keyword.Test_iskeyword.test_changing_the_softkwlist_does_not_affect_issoftkeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_changing_the_softkwlist_does_not_affect_issoftkeyword
"""Auto-ported test: Test_iskeyword::test_changing_the_softkwlist_does_not_affect_issoftkeyword (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---
oldlist = keyword.softkwlist
pass
keyword.softkwlist = ['foo', 'bar', 'spam', 'egs', 'case']

assert not keyword.issoftkeyword('spam')
print("Test_iskeyword::test_changing_the_softkwlist_does_not_affect_issoftkeyword: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_changing_the_softkwlist_does_not_affect_issoftkeyword: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_keywords_are_sorted.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_keywords_are_sorted() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_keywords_are_sorted"
# subject = "cpython.test_keyword.Test_iskeyword.test_keywords_are_sorted"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_keywords_are_sorted
"""Auto-ported test: Test_iskeyword::test_keywords_are_sorted (CPython 3.12 oracle)."""


import keyword


assert sorted(keyword.kwlist) == keyword.kwlist

print("Test_iskeyword::test_keywords_are_sorted: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_keywords_are_sorted: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_soft_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_soft_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_soft_keywords"
# subject = "cpython.test_keyword.Test_iskeyword.test_soft_keywords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_soft_keywords
"""Auto-ported test: Test_iskeyword::test_soft_keywords (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---

assert 'type' in keyword.softkwlist

assert 'match' in keyword.softkwlist

assert 'case' in keyword.softkwlist

assert '_' in keyword.softkwlist
print("Test_iskeyword::test_soft_keywords: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_soft_keywords: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_softkeywords_are_sorted.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_softkeywords_are_sorted() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_softkeywords_are_sorted"
# subject = "cpython.test_keyword.Test_iskeyword.test_softkeywords_are_sorted"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_softkeywords_are_sorted
"""Auto-ported test: Test_iskeyword::test_softkeywords_are_sorted (CPython 3.12 oracle)."""


import keyword


assert sorted(keyword.softkwlist) == keyword.softkwlist

print("Test_iskeyword::test_softkeywords_are_sorted: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_softkeywords_are_sorted: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_true_is_a_keyword.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_true_is_a_keyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_true_is_a_keyword"
# subject = "cpython.test_keyword.Test_iskeyword.test_true_is_a_keyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_true_is_a_keyword
"""Auto-ported test: Test_iskeyword::test_true_is_a_keyword (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---

assert keyword.iskeyword('True')
print("Test_iskeyword::test_true_is_a_keyword: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_true_is_a_keyword: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/keyword/test_iskeyword__test_uppercase_true_is_not_a_keyword.py`.
#[test]
fn test_gen_behavior_std_libs_keyword_test_iskeyword__test_uppercase_true_is_not_a_keyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_uppercase_true_is_not_a_keyword"
# subject = "cpython.test_keyword.Test_iskeyword.test_uppercase_true_is_not_a_keyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_uppercase_true_is_not_a_keyword
"""Auto-ported test: Test_iskeyword::test_uppercase_true_is_not_a_keyword (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---

assert not keyword.iskeyword('TRUE')
print("Test_iskeyword::test_uppercase_true_is_not_a_keyword: ok")
"###);
    assert_output(&out, r###"Test_iskeyword::test_uppercase_true_is_not_a_keyword: ok
"###);
}
