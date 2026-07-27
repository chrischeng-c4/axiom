use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/bytes_patterns_match_like_str.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_bytes_patterns_match_like_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "bytes_patterns_match_like_str"
# subject = "fnmatch.fnmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatch: all-bytes args match like the str version: fnmatch(b'test.py', b'*.py') is True, fnmatch(b'test.rs', b'*.py') is False, fnmatchcase stays case-sensitive on bytes"""
import fnmatch

# fnmatch with all-bytes args matches like the str version (lowercase inputs
# avoid the OS-dependent case fold of fnmatch).
assert fnmatch.fnmatch(b"test.py", b"*.py") is True, "bytes fnmatch suffix"
assert fnmatch.fnmatch(b"test.rs", b"*.py") is False, "bytes fnmatch no match"

# fnmatchcase with bytes is strictly case-sensitive.
assert fnmatch.fnmatchcase(b"test.PY", b"*.PY") is True, "bytes upper suffix"
assert fnmatch.fnmatchcase(b"test.py", b"*.PY") is False, "bytes case mismatch"

print("bytes_patterns_match_like_str OK")
"###);
    assert_output(&out, r###"bytes_patterns_match_like_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/char_class_inclusive_range.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_char_class_inclusive_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "char_class_inclusive_range"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: [a-z] matches an inclusive lowercase range: both endpoints 'a' and 'z' match, a digit '0' does not"""
import fnmatch

assert fnmatch.fnmatchcase("file_a.txt", "file_[a-z].txt"), "lowercase range start"
assert fnmatch.fnmatchcase("file_z.txt", "file_[a-z].txt"), "end of range"
assert not fnmatch.fnmatchcase("file_0.txt", "file_[a-z].txt"), "digit out of range"

print("char_class_inclusive_range OK")
"###);
    assert_output(&out, r###"char_class_inclusive_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/char_class_negated.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_char_class_negated() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "char_class_negated"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: [!a-z] is a negated class: a digit '0' matches, a lowercase letter 'a' does not"""
import fnmatch

assert fnmatch.fnmatchcase("file_0.txt", "file_[!a-z].txt"), "negated class matches digit"
assert not fnmatch.fnmatchcase("file_a.txt", "file_[!a-z].txt"), "negated class excludes letter"

print("char_class_negated OK")
"###);
    assert_output(&out, r###"char_class_negated OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/char_class_set_membership.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_char_class_set_membership() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "char_class_set_membership"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: [abc] matches set membership: '[abc]at' matches 'cat' but not 'xat'; the negated '[!abc]at' matches 'xat'"""
import fnmatch

assert fnmatch.fnmatchcase("cat", "[abc]at"), "set member matches"
assert not fnmatch.fnmatchcase("xat", "[abc]at"), "non-member excluded"
assert fnmatch.fnmatchcase("xat", "[!abc]at"), "negated set matches non-member"

print("char_class_set_membership OK")
"###);
    assert_output(&out, r###"char_class_set_membership OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/filter_all_matches_returns_all.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_filter_all_matches_returns_all() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_all_matches_returns_all"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.filter: filter returns the whole list (a new equal list) when every name matches"""
import fnmatch

_all = ["a.txt", "b.txt", "c.txt"]
_out = fnmatch.filter(_all, "*.txt")
assert _out == _all, f"all match -> {_out!r}"

print("filter_all_matches_returns_all OK")
"###);
    assert_output(&out, r###"filter_all_matches_returns_all OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/filter_bytes_returns_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_filter_bytes_returns_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_bytes_returns_bytes"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.filter: filter with bytes names + bytes pattern keeps the matching bytes objects (every result element is bytes); filter(['Python',...], 'P*') with str is unaffected"""
import fnmatch

# filter with bytes names + bytes pattern keeps the matching bytes objects.
_b = fnmatch.filter([b"Python", b"Ruby", b"Perl", b"Tcl"], b"P*")
assert _b == [b"Python", b"Perl"], f"bytes filter = {_b!r}"
assert all(isinstance(x, bytes) for x in _b), "filter returns bytes elements"

# The str path is unaffected by the bytes path.
_s = fnmatch.filter(["Python", "Ruby", "Perl", "Tcl"], "P*")
assert _s == ["Python", "Perl"], f"str filter = {_s!r}"

print("filter_bytes_returns_bytes OK")
"###);
    assert_output(&out, r###"filter_bytes_returns_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/filter_keeps_matching_in_order.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_filter_keeps_matching_in_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_keeps_matching_in_order"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.filter: filter returns the matching names in input order; mixed list filtered by '*.txt' keeps only the .txt names in their original positions"""
import fnmatch

_names = ["b.txt", "a.py", "c.txt", "d.txt", "e.py"]
_filtered = fnmatch.filter(_names, "*.txt")
assert _filtered == ["b.txt", "c.txt", "d.txt"], f"filter order = {_filtered!r}"

print("filter_keeps_matching_in_order OK")
"###);
    assert_output(&out, r###"filter_keeps_matching_in_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/filter_no_matches_empty_list.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_filter_no_matches_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_no_matches_empty_list"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.filter: filter returns [] when nothing matches the pattern"""
import fnmatch

assert fnmatch.filter(["a.py", "b.rs"], "*.txt") == [], "no matches yields empty list"

print("filter_no_matches_empty_list OK")
"###);
    assert_output(&out, r###"filter_no_matches_empty_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/filter_prefix_pattern.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_filter_prefix_pattern() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_prefix_pattern"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.filter: filter(['Python','Ruby','Perl','Tcl'], 'P*') keeps the names starting with P in order: ['Python','Perl']"""
import fnmatch

_out = fnmatch.filter(["Python", "Ruby", "Perl", "Tcl"], "P*")
assert _out == ["Python", "Perl"], f"prefix filter = {_out!r}"

print("filter_prefix_pattern OK")
"###);
    assert_output(&out, r###"filter_prefix_pattern OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/fnmatch_test_case__test_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_fnmatch_test_case__test_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_bytes"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_bytes
"""Auto-ported test: FnmatchTestCase::test_bytes (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---
def check_match(filename, pattern, should_match=True, fn=fnmatch):
    if should_match:

        assert fn(filename, pattern)
    else:

        assert not fn(filename, pattern)
check_match(b'test', b'te*')
check_match(b'test\xff', b'te*\xff')
check_match(b'foo\nbar', b'foo*')
print("FnmatchTestCase::test_bytes: ok")
"###);
    assert_output(&out, r###"FnmatchTestCase::test_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/fnmatchcase_is_case_sensitive.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_fnmatchcase_is_case_sensitive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatchcase_is_case_sensitive"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: fnmatchcase is strictly case-sensitive regardless of OS: 'README.MD' matches '*.MD', but 'readme.md' does not match '*.MD' (only '*.md')"""
import fnmatch

assert fnmatch.fnmatchcase("README.MD", "*.MD"), "upper suffix matches upper pattern"
assert not fnmatch.fnmatchcase("readme.md", "*.MD"), "case mismatch does not match"
assert fnmatch.fnmatchcase("readme.md", "*.md"), "lower suffix matches lower pattern"

print("fnmatchcase_is_case_sensitive OK")
"###);
    assert_output(&out, r###"fnmatchcase_is_case_sensitive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/question_matches_exactly_one.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_question_matches_exactly_one() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "question_matches_exactly_one"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: ? matches exactly one character: 'a?c' matches 'abc' but not 'ac' (too few) nor 'aXXc' (too many)"""
import fnmatch

assert fnmatch.fnmatchcase("abc", "a?c"), "? matches one"
assert not fnmatch.fnmatchcase("ac", "a?c"), "? requires exactly one"
assert not fnmatch.fnmatchcase("aXXc", "a?c"), "? does not match two"

print("question_matches_exactly_one OK")
"###);
    assert_output(&out, r###"question_matches_exactly_one OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/star_crosses_path_separators.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_star_crosses_path_separators() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "star_crosses_path_separators"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatchcase: fnmatch is string-level not path-level: * spans '/' so 'a/b/c.txt' matches '*.txt' and '*/*.txt'"""
import fnmatch

# Unlike glob, fnmatch is string-level — * matches the '/' separator too.
assert fnmatch.fnmatchcase("a/b/c.txt", "*.txt"), "* crosses path separators"
assert fnmatch.fnmatchcase("a/b/c.txt", "*/*.txt"), "nested glob across separators"

print("star_crosses_path_separators OK")
"###);
    assert_output(&out, r###"star_crosses_path_separators OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/star_matches_any_run.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_star_matches_any_run() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "star_matches_any_run"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatchcase: * matches any run of characters including zero and the empty string; '*.txt' matches multi-segment names and a leading dot, and bare '*' matches the empty string"""
import fnmatch

# * spans any run of characters, including across dots.
assert fnmatch.fnmatchcase("hello.world.txt", "*.txt"), "* multi-segment"
# '*.txt' still requires the literal dot.
assert fnmatch.fnmatchcase("txt", "*.txt") is False, "*.txt requires dot"
assert fnmatch.fnmatchcase(".txt", "*.txt"), "*.txt matches .txt"
# * matches the empty string and a leading-dot name.
assert fnmatch.fnmatchcase(".hidden", "*"), "* matches leading dot"
assert fnmatch.fnmatchcase("", "*"), "* matches empty"

print("star_matches_any_run OK")
"###);
    assert_output(&out, r###"star_matches_any_run OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/translate_bracket_expressions.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_translate_bracket_expressions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_bracket_expressions"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.translate: translate bracket rules: [abc]->[abc], []]->[]] (literal close-bracket first), [!x]->[^x], [^x]->[\\^x] (literal caret escaped), [x->\\[x (unterminated bracket is literal)"""
import fnmatch

assert fnmatch.translate("[abc]") == "(?s:[abc])\\Z", "class"
assert fnmatch.translate("[]]") == "(?s:[]])\\Z", "literal close-bracket first"
assert fnmatch.translate("[!x]") == "(?s:[^x])\\Z", "negated class -> caret"
assert fnmatch.translate("[^x]") == "(?s:[\\^x])\\Z", "literal caret escaped"
assert fnmatch.translate("[x") == "(?s:\\[x)\\Z", "unterminated bracket is literal"

print("translate_bracket_expressions OK")
"###);
    assert_output(&out, r###"translate_bracket_expressions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/translate_collapses_consecutive_stars.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_translate_collapses_consecutive_stars() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_collapses_consecutive_stars"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.translate: translate squashes runs of consecutive stars to a single .*: '*********'->(?s:.*)\\Z, 'A*********'->(?s:A.*)\\Z, '*********A'->(?s:.*A)\\Z, 'A*********?[?]?'->(?s:A.*.[?].)\\Z"""
import fnmatch

assert fnmatch.translate("*********") == "(?s:.*)\\Z", "all stars collapse"
assert fnmatch.translate("A*********") == "(?s:A.*)\\Z", "leading literal"
assert fnmatch.translate("*********A") == "(?s:.*A)\\Z", "trailing literal"
assert fnmatch.translate("A*********?[?]?") == "(?s:A.*.[?].)\\Z", "stars + ? + class"

print("translate_collapses_consecutive_stars OK")
"###);
    assert_output(&out, r###"translate_collapses_consecutive_stars OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/translate_returns_anchored_regex.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_translate_returns_anchored_regex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_returns_anchored_regex"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.translate: translate returns a str regex wrapped in (?s:...)\\Z that is anchored to the full string: the compiled regex matches 'script.py' but not 'script.py.bak'"""
import re
import fnmatch

_re_str = fnmatch.translate("*.py")
assert isinstance(_re_str, str), f"translate type = {type(_re_str)!r}"
_pat = re.compile(_re_str)
assert _pat.match("script.py"), "anchored regex matches full string"
assert not _pat.match("script.py.bak"), "anchored at end rejects trailing text"

print("translate_returns_anchored_regex OK")
"###);
    assert_output(&out, r###"translate_returns_anchored_regex OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/translate_single_metachar_substitutions.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_translate_single_metachar_substitutions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_single_metachar_substitutions"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.translate: translate maps * -> .*, ? -> ., and pins the exact 3.12 strings for '*', '?', 'a?b*', '*.txt'"""
import fnmatch

assert fnmatch.translate("*") == "(?s:.*)\\Z", "star"
assert fnmatch.translate("?") == "(?s:.)\\Z", "question"
assert fnmatch.translate("a?b*") == "(?s:a.b.*)\\Z", "mixed metachars"
assert fnmatch.translate("*.txt") == "(?s:.*\\.txt)\\Z", "literal dot escaped"

print("translate_single_metachar_substitutions OK")
"###);
    assert_output(&out, r###"translate_single_metachar_substitutions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/fnmatch/unterminated_bracket_no_match.py`.
#[test]
fn test_gen_behavior_std_libs_fnmatch_unterminated_bracket_no_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "unterminated_bracket_no_match"
# subject = "fnmatch.fnmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatch: a malformed pattern is forgiving, not an error: an unterminated bracket '[abc' does not raise and treats the bracket literally so it does not match 'a'"""
import fnmatch

# A bad bracket is not an error; it is treated literally and simply does not
# match. fnmatch is forgiving.
_result = fnmatch.fnmatch("a", "[abc")
assert _result is False, f"unterminated bracket no match = {_result!r}"

print("unterminated_bracket_no_match OK")
"###);
    assert_output(&out, r###"unterminated_bracket_no_match OK
"###);
}
