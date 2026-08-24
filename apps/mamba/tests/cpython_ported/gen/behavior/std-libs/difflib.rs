use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/difflib/autojunk_false_keeps_long_match.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_autojunk_false_keeps_long_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "autojunk_false_keeps_long_match"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: with autojunk=False a 200-char common run is NOT treated as junk, so ratio of 'a'*200+'b' vs 'a'*200+'c' is > 0.99"""
import difflib

_sm = difflib.SequenceMatcher(
    autojunk=False, a=list("a" * 200 + "b"), b=list("a" * 200 + "c"))
_ratio = _sm.ratio()
assert _ratio > 0.99, f"autojunk=False ratio = {_ratio!r}"  # almost identical
print("autojunk_false_keeps_long_match OK")
"###);
    assert_output(&out, r###"autojunk_false_keeps_long_match OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/autojunk_true_marks_popular.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_autojunk_true_marks_popular() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "autojunk_true_marks_popular"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: default autojunk=True marks the repeated 'b' as bpopular for a 200+ sequence, collapsing ratio toward 0"""
import difflib

_seq1 = "b" * 200
_seq2 = "a" + "b" * 200
_sm = difflib.SequenceMatcher(None, _seq1, _seq2)  # default autojunk=True
assert round(_sm.ratio(), 3) == 0.0, f"ratio = {_sm.ratio()!r}"
assert _sm.bpopular == {"b"}, f"bpopular = {_sm.bpopular!r}"
print("autojunk_true_marks_popular OK")
"###);
    assert_output(&out, r###"autojunk_true_marks_popular OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/context_diff_star_markers.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_context_diff_star_markers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "context_diff_star_markers"
# subject = "difflib.context_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.context_diff: context_diff emits '*** 1,3 ****' (from-range) and '--- 1,3 ----' (to-range) markers for a 3-line file with one change"""
import difflib

_a = "a\nb\nc\n".splitlines(keepends=True)
_b = "a\nX\nc\n".splitlines(keepends=True)
_cd = list(difflib.context_diff(_a, _b, lineterm=""))
assert "*** 1,3 ****" in _cd, f"context from-range = {_cd!r}"
assert "--- 1,3 ----" in _cd, f"context to-range = {_cd!r}"
print("context_diff_star_markers OK")
"###);
    assert_output(&out, r###"context_diff_star_markers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/context_diff_tab_delimited_filedates.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_context_diff_tab_delimited_filedates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "context_diff_tab_delimited_filedates"
# subject = "difflib.context_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.context_diff: with filedates the context_diff '***'/'---' file headers are tab-separated 'name\\tdate'"""
import difflib

_cd = list(difflib.context_diff(
    ["one"], ["two"], "Original", "Current",
    "2005-01-26 23:30:50", "2010-04-02 10:20:52", lineterm=""))
assert _cd[0] == "*** Original\t2005-01-26 23:30:50", f"from header = {_cd[0]!r}"
assert _cd[1] == "--- Current\t2010-04-02 10:20:52", f"to header = {_cd[1]!r}"
print("context_diff_tab_delimited_filedates OK")
"###);
    assert_output(&out, r###"context_diff_tab_delimited_filedates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/differ_added_tab_hint.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_differ_added_tab_hint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "differ_added_tab_hint"
# subject = "difflib.Differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.Differ: Differ().compare emits a '? ' guide line whose '--'/'+' markers point at the changed (added-tab) columns"""
import difflib

_diff = list(difflib.Differ().compare(["\tI am a buggy"], ["\t\tI am a bug"]))
assert _diff[0] == "- \tI am a buggy", f"line0 = {_diff[0]!r}"
assert _diff[1] == "? \t          --\n", f"line1 = {_diff[1]!r}"
assert _diff[2] == "+ \t\tI am a bug", f"line2 = {_diff[2]!r}"
assert _diff[3] == "? +\n", f"line3 = {_diff[3]!r}"
print("differ_added_tab_hint OK")
"###);
    assert_output(&out, r###"differ_added_tab_hint OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/differ_compare_basic_markers.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_differ_compare_basic_markers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "differ_compare_basic_markers"
# subject = "difflib.Differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.Differ: Differ().compare marks each line: '  ' unchanged, '- ' removed, '+ ' added"""
import difflib

_diff = list(difflib.Differ().compare(["foo\n", "bar\n"], ["foo\n", "baz\n"]))
assert "  foo\n" in _diff, f"unchanged marker missing: {_diff!r}"
assert "- bar\n" in _diff, f"removed marker missing: {_diff!r}"
assert "+ baz\n" in _diff, f"added marker missing: {_diff!r}"
print("differ_compare_basic_markers OK")
"###);
    assert_output(&out, r###"differ_compare_basic_markers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/differ_hint_indented_with_tabs.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_differ_hint_indented_with_tabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "differ_hint_indented_with_tabs"
# subject = "difflib.Differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.Differ: Differ guide-line indentation expands tabs so the '+' marker lands directly under the changed column"""
import difflib

_diff = list(difflib.Differ().compare(["\t \t \t^"], ["\t \t \t^\n"]))
assert _diff[0] == "- \t \t \t^", f"line0 = {_diff[0]!r}"
assert _diff[1] == "+ \t \t \t^\n", f"line1 = {_diff[1]!r}"
assert _diff[2] == "? \t \t \t +\n", f"line2 = {_diff[2]!r}"
print("differ_hint_indented_with_tabs OK")
"###);
    assert_output(&out, r###"differ_hint_indented_with_tabs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_close_matches_empty_when_below_cutoff.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_close_matches_empty_when_below_cutoff() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_close_matches_empty_when_below_cutoff"
# subject = "difflib.get_close_matches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches with a high cutoff and no qualifying word returns []"""
import difflib

_no_match = difflib.get_close_matches(
    "zzzzz", ["apple", "banana", "cherry"], cutoff=0.9)
assert _no_match == [], f"no match = {_no_match!r}"
print("get_close_matches_empty_when_below_cutoff OK")
"###);
    assert_output(&out, r###"get_close_matches_empty_when_below_cutoff OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_close_matches_finds_typo.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_close_matches_finds_typo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_close_matches_finds_typo"
# subject = "difflib.get_close_matches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches('aple', ['apple', 'apricot', 'banana', 'mango']) ranks 'apple' first as the closest word"""
import difflib

_words = ["apple", "apricot", "banana", "mango"]
_matches = difflib.get_close_matches("aple", _words)
assert isinstance(_matches, list), f"close_matches type = {type(_matches)!r}"
assert _matches[0] == "apple", f"closest match ranked first = {_matches!r}"
print("get_close_matches_finds_typo OK")
"###);
    assert_output(&out, r###"get_close_matches_finds_typo OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_close_matches_respects_n_and_cutoff.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_close_matches_respects_n_and_cutoff() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_close_matches_respects_n_and_cutoff"
# subject = "difflib.get_close_matches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches('helo', words, n=3, cutoff=0.5) returns at most 3 results and includes a real near-match like 'hello' or 'help'"""
import difflib

_words = ["help", "hello", "world", "helm", "hero"]
_m = difflib.get_close_matches("helo", _words, n=3, cutoff=0.5)
assert isinstance(_m, list), f"close_matches type = {type(_m)!r}"
assert len(_m) <= 3, f"at most n=3 results = {_m!r}"
assert "hello" in _m or "help" in _m, f"close match found: {_m!r}"
print("get_close_matches_respects_n_and_cutoff OK")
"###);
    assert_output(&out, r###"get_close_matches_respects_n_and_cutoff OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_grouped_opcodes_empty_is_empty.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_grouped_opcodes_empty_is_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_grouped_opcodes_empty_is_empty"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: get_grouped_opcodes() over two empty sequences yields nothing (next() raises StopIteration)"""
import difflib

_grp = difflib.SequenceMatcher(None, [], []).get_grouped_opcodes()
_raised = False
try:
    next(_grp)
except StopIteration:
    _raised = True
assert _raised, "expected empty grouped opcodes"
print("get_grouped_opcodes_empty_is_empty OK")
"###);
    assert_output(&out, r###"get_grouped_opcodes_empty_is_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_opcodes_are_five_tuples.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_opcodes_are_five_tuples() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_are_five_tuples"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: get_opcodes returns 5-tuples whose tags are drawn from {equal, insert, delete, replace}"""
import difflib

_sm = difflib.SequenceMatcher(None, "hello", "helo")
_ops = _sm.get_opcodes()
assert isinstance(_ops, list), f"opcodes type = {type(_ops)!r}"
assert all(len(op) == 5 for op in _ops), "opcodes are 5-tuples"
_tags = {op[0] for op in _ops}
assert _tags <= {"equal", "insert", "delete", "replace"}, f"valid tags = {_tags!r}"
print("get_opcodes_are_five_tuples OK")
"###);
    assert_output(&out, r###"get_opcodes_are_five_tuples OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_opcodes_reconstruct_target.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_opcodes_reconstruct_target() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_reconstruct_target"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: applying the opcodes of ABCDE->ACE (taking equal/insert/replace from b, skipping delete) reconstructs 'ACE'"""
import difflib

_sm = difflib.SequenceMatcher(None, "ABCDE", "ACE")
_result = []
for _tag, _i1, _i2, _j1, _j2 in _sm.get_opcodes():
    if _tag == "equal":
        _result.extend("ABCDE"[_i1:_i2])
    elif _tag == "insert":
        _result.extend("ACE"[_j1:_j2])
    elif _tag == "replace":
        _result.extend("ACE"[_j1:_j2])
    # "delete" just skips the old span.
assert "".join(_result) == "ACE", f"opcode apply = {''.join(_result)!r}"
print("get_opcodes_reconstruct_target OK")
"###);
    assert_output(&out, r###"get_opcodes_reconstruct_target OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_opcodes_single_delete.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_opcodes_single_delete() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_single_delete"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: a single middle deletion yields exactly [equal 0,40,0,40; delete 40,41,40,40; equal 41,81,40,80] and ratio ~= 0.994"""
import difflib

_sm = difflib.SequenceMatcher(None, "a" * 40 + "c" + "b" * 40, "a" * 40 + "b" * 40)
assert round(_sm.ratio(), 3) == 0.994, f"ratio = {_sm.ratio()!r}"
assert list(_sm.get_opcodes()) == [
    ("equal", 0, 40, 0, 40),
    ("delete", 40, 41, 40, 40),
    ("equal", 41, 81, 40, 80),
], f"opcodes = {list(_sm.get_opcodes())!r}"
print("get_opcodes_single_delete OK")
"###);
    assert_output(&out, r###"get_opcodes_single_delete OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/get_opcodes_single_insert.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_get_opcodes_single_insert() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_single_insert"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: a single leading insertion ('b'*100 vs 'a'+'b'*100) yields exactly [insert 0,0,0,1; equal 0,100,1,101] and ratio ~= 0.995"""
import difflib

_sm = difflib.SequenceMatcher(None, "b" * 100, "a" + "b" * 100)
assert round(_sm.ratio(), 3) == 0.995, f"ratio = {_sm.ratio()!r}"
assert list(_sm.get_opcodes()) == [
    ("insert", 0, 0, 0, 1),
    ("equal", 0, 100, 1, 101),
], f"opcodes = {list(_sm.get_opcodes())!r}"
assert _sm.bpopular == set(), f"bpopular = {_sm.bpopular!r}"
print("get_opcodes_single_insert OK")
"###);
    assert_output(&out, r###"get_opcodes_single_insert OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/htmldiff_default_charset_utf8.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_htmldiff_default_charset_utf8() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "htmldiff_default_charset_utf8"
# subject = "difflib.HtmlDiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.HtmlDiff: HtmlDiff().make_file defaults the <meta> charset to utf-8"""
import difflib

_from = ["Beautiful is better than ugly", "Explicit is better"]
_to = ["Beautiful is better than nice", "Explicit is best"]
_default = difflib.HtmlDiff().make_file(_from, _to)
assert 'content="text/html; charset=utf-8"' in _default, "default charset utf-8"
print("htmldiff_default_charset_utf8 OK")
"###);
    assert_output(&out, r###"htmldiff_default_charset_utf8 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/htmldiff_explicit_charset.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_htmldiff_explicit_charset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "htmldiff_explicit_charset"
# subject = "difflib.HtmlDiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.HtmlDiff: an explicit charset='iso-8859-1' flows into the make_file <meta> tag"""
import difflib

_from = ["Beautiful is better than ugly", "Explicit is better"]
_to = ["Beautiful is better than nice", "Explicit is best"]
_iso = difflib.HtmlDiff().make_file(_from, _to, charset="iso-8859-1")
assert 'content="text/html; charset=iso-8859-1"' in _iso, "iso-8859-1 charset"
print("htmldiff_explicit_charset OK")
"###);
    assert_output(&out, r###"htmldiff_explicit_charset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/htmldiff_nonascii_escaped_under_usascii.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_htmldiff_nonascii_escaped_under_usascii() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "htmldiff_nonascii_escaped_under_usascii"
# subject = "difflib.HtmlDiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.HtmlDiff: non-ASCII input under charset='us-ascii' is escaped to numeric character references (e.g. &#305;)"""
import difflib

_nonascii_from = ["Explicit is better than ımplıcıt"]
_nonascii_to = ["Explicit is better than implicit"]
_usascii = difflib.HtmlDiff().make_file(
    _nonascii_from, _nonascii_to, charset="us-ascii")
assert 'content="text/html; charset=us-ascii"' in _usascii, "us-ascii charset"
assert "&#305;" in _usascii, "non-ascii escaped to numeric entity (&#305;)"
print("htmldiff_nonascii_escaped_under_usascii OK")
"###);
    assert_output(&out, r###"htmldiff_nonascii_escaped_under_usascii OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/is_character_junk_space_tab_only.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_is_character_junk_space_tab_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "is_character_junk_space_tab_only"
# subject = "difflib.IS_CHARACTER_JUNK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.IS_CHARACTER_JUNK: IS_CHARACTER_JUNK is True only for ' ' and '\\t'; False for 'a', '#', newline, formfeed, carriage-return, vertical-tab"""
import difflib

for _ch in (" ", "\t"):
    assert difflib.IS_CHARACTER_JUNK(_ch), f"char junk true: {_ch!r}"
for _ch in ("a", "#", "\n", "\x0c", "\r", "\x0b"):
    assert not difflib.IS_CHARACTER_JUNK(_ch), f"char junk false: {_ch!r}"
print("is_character_junk_space_tab_only OK")
"###);
    assert_output(&out, r###"is_character_junk_space_tab_only OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/is_line_junk_blank_or_single_hash.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_is_line_junk_blank_or_single_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "is_line_junk_blank_or_single_hash"
# subject = "difflib.IS_LINE_JUNK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.IS_LINE_JUNK: IS_LINE_JUNK is True for a line of only blanks and/or a single '#'; False for '##', non-blank text, etc."""
import difflib

for _line in ("#", "  ", " #", "# ", " # ", ""):
    assert difflib.IS_LINE_JUNK(_line), f"line junk true: {_line!r}"
for _line in ("##", " ##", "## ", "abc ", "abc #", "Mr. Moose is up!"):
    assert not difflib.IS_LINE_JUNK(_line), f"line junk false: {_line!r}"
print("is_line_junk_blank_or_single_hash OK")
"###);
    assert_output(&out, r###"is_line_junk_blank_or_single_hash OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/matching_blocks_cached_and_sentinel.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_matching_blocks_cached_and_sentinel() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "matching_blocks_cached_and_sentinel"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: get_matching_blocks for 'abxcd' vs 'abcd' is cached/idempotent; block[0] is Match(a=0,b=0,size=2) and the trailing sentinel block has size 0"""
import difflib

_sm = difflib.SequenceMatcher(None, "abxcd", "abcd")
_first = _sm.get_matching_blocks()
_second = _sm.get_matching_blocks()
assert _first == _second, "matching blocks cached/idempotent"
assert _second[0].a == 0 and _second[0].b == 0, f"block[0] a/b = {_second[0]!r}"
assert _second[0].size == 2, f"block[0].size = {_second[0].size!r}"
assert _second[-1].size == 0, "sentinel block has size 0"
print("matching_blocks_cached_and_sentinel OK")
"###);
    assert_output(&out, r###"matching_blocks_cached_and_sentinel OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/ndiff_marks_added_removed_unchanged.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_ndiff_marks_added_removed_unchanged() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ndiff_marks_added_removed_unchanged"
# subject = "difflib.ndiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.ndiff: ndiff prefixes lines with '- ' (removed), '+ ' (added) and '  ' (unchanged)"""
import difflib

_nd = list(difflib.ndiff(["foo\n", "bar\n"], ["foo\n", "baz\n"]))
assert any(line.startswith("- bar") for line in _nd), f"- line missing: {_nd!r}"
assert any(line.startswith("+ baz") for line in _nd), f"+ line missing: {_nd!r}"
assert any(line.startswith("  foo") for line in _nd), f"unchanged line missing: {_nd!r}"
print("ndiff_marks_added_removed_unchanged OK")
"###);
    assert_output(&out, r###"ndiff_marks_added_removed_unchanged OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/ratio_disjoint_is_zero.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_ratio_disjoint_is_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_disjoint_is_zero"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio() of two strings with no common characters is exactly 0.0"""
import difflib

_sm = difflib.SequenceMatcher(None, "abc", "xyz")
assert _sm.ratio() == 0.0, f"different ratio = {_sm.ratio()!r}"
print("ratio_disjoint_is_zero OK")
"###);
    assert_output(&out, r###"ratio_disjoint_is_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/ratio_empty_sequences_is_one.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_ratio_empty_sequences_is_one() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_empty_sequences_is_one"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio/quick_ratio/real_quick_ratio of two empty sequences are all 1.0 (vacuously identical)"""
import difflib

_sm = difflib.SequenceMatcher(None, [], [])
assert _sm.ratio() == 1.0, f"empty ratio = {_sm.ratio()!r}"
assert _sm.quick_ratio() == 1.0, f"empty quick_ratio = {_sm.quick_ratio()!r}"
assert _sm.real_quick_ratio() == 1.0, f"empty real_quick_ratio = {_sm.real_quick_ratio()!r}"
print("ratio_empty_sequences_is_one OK")
"###);
    assert_output(&out, r###"ratio_empty_sequences_is_one OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/ratio_identical_is_one.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_ratio_identical_is_one() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_identical_is_one"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio() of two identical strings is exactly 1.0"""
import difflib

_sm = difflib.SequenceMatcher(None, "abc", "abc")
assert _sm.ratio() == 1.0, f"identical ratio = {_sm.ratio()!r}"
print("ratio_identical_is_one OK")
"###);
    assert_output(&out, r###"ratio_identical_is_one OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/ratio_partial_match_fraction.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_ratio_partial_match_fraction() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_partial_match_fraction"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio() of 'hello' vs 'helo' is 2*M/T = 8/9; quick_ratio is an upper bound >= ratio"""
import difflib

_sm = difflib.SequenceMatcher(None, "hello", "helo")
_r = _sm.ratio()
# 4 matched chars out of (5 + 4) total -> 2*4/9
assert _r == 2 * 4 / (5 + 4), f"ratio = {_r!r}"
assert 0.0 <= _r <= 1.0, f"ratio range = {_r!r}"
assert _r > 0.5, f"ratio > 0.5 = {_r!r}"
# quick_ratio is an upper bound on ratio.
_qr = _sm.quick_ratio()
assert _qr >= _r, f"quick_ratio {_qr!r} >= ratio {_r!r}"
print("ratio_partial_match_fraction OK")
"###);
    assert_output(&out, r###"ratio_partial_match_fraction OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/restore_recovers_original_sequences.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_restore_recovers_original_sequences() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "restore_recovers_original_sequences"
# subject = "difflib.restore"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.restore: restore(ndiff_output, 1) rebuilds the from-sequence and restore(..., 2) rebuilds the to-sequence"""
import difflib

_from = ["one\n", "two\n", "three\n"]
_to = ["ore\n", "tree\n", "emu\n"]
_diff = list(difflib.ndiff(_from, _to))
assert list(difflib.restore(_diff, 1)) == _from, "restore(diff, 1) rebuilds from-seq"
assert list(difflib.restore(_diff, 2)) == _to, "restore(diff, 2) rebuilds to-seq"
print("restore_recovers_original_sequences OK")
"###);
    assert_output(&out, r###"restore_recovers_original_sequences OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_bytes__test_byte_content.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_bytes__test_byte_content() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_bytes__test_byte_content"
# subject = "cpython.test_difflib.TestBytes.test_byte_content"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestBytes::test_byte_content
"""Auto-ported test: TestBytes::test_byte_content (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
def _assert_type_error(msg, generator, *args):
    try:
        list(generator(*args))
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import types as _types_aR
        ctx = _types_aR.SimpleNamespace(exception=_aR_e)

    assert msg == str(ctx.exception)

def check(diff):
    diff = list(diff)
    for line in diff:

        assert isinstance(line, bytes)
a = [b'hello', b'andr\xe9']
b = [b'hello', b'andr\xc3\xa9']
unified = difflib.unified_diff
context = difflib.context_diff
check = check
check(difflib.diff_bytes(unified, a, a))
check(difflib.diff_bytes(unified, a, b))
check(difflib.diff_bytes(unified, a, a, b'a', b'a'))
check(difflib.diff_bytes(unified, a, b, b'a', b'b'))
check(difflib.diff_bytes(unified, a, a, b'a', b'a', b'2005', b'2013'))
check(difflib.diff_bytes(unified, a, b, b'a', b'b', b'2005', b'2013'))
check(difflib.diff_bytes(context, a, a))
check(difflib.diff_bytes(context, a, b))
check(difflib.diff_bytes(context, a, a, b'a', b'a'))
check(difflib.diff_bytes(context, a, b, b'a', b'b'))
check(difflib.diff_bytes(context, a, a, b'a', b'a', b'2005', b'2013'))
check(difflib.diff_bytes(context, a, b, b'a', b'b', b'2005', b'2013'))
print("TestBytes::test_byte_content: ok")
"###);
    assert_output(&out, r###"TestBytes::test_byte_content: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_junk_ap_is__test_is_character_junk_false.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_junk_ap_is__test_is_character_junk_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_junk_ap_is__test_is_character_junk_false"
# subject = "cpython.test_difflib.TestJunkAPIs.test_is_character_junk_false"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestJunkAPIs::test_is_character_junk_false
"""Auto-ported test: TestJunkAPIs::test_is_character_junk_false (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
for char in ['a', '#', '\n', '\x0c', '\r', '\x0b']:

    assert not difflib.IS_CHARACTER_JUNK(char)
print("TestJunkAPIs::test_is_character_junk_false: ok")
"###);
    assert_output(&out, r###"TestJunkAPIs::test_is_character_junk_false: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_junk_ap_is__test_is_character_junk_true.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_junk_ap_is__test_is_character_junk_true() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_junk_ap_is__test_is_character_junk_true"
# subject = "cpython.test_difflib.TestJunkAPIs.test_is_character_junk_true"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestJunkAPIs::test_is_character_junk_true
"""Auto-ported test: TestJunkAPIs::test_is_character_junk_true (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
for char in [' ', '\t']:

    assert difflib.IS_CHARACTER_JUNK(char)
print("TestJunkAPIs::test_is_character_junk_true: ok")
"###);
    assert_output(&out, r###"TestJunkAPIs::test_is_character_junk_true: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_junk_ap_is__test_is_line_junk_false.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_junk_ap_is__test_is_line_junk_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_junk_ap_is__test_is_line_junk_false"
# subject = "cpython.test_difflib.TestJunkAPIs.test_is_line_junk_false"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestJunkAPIs::test_is_line_junk_false
"""Auto-ported test: TestJunkAPIs::test_is_line_junk_false (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
for line in ['##', ' ##', '## ', 'abc ', 'abc #', 'Mr. Moose is up!']:

    assert not difflib.IS_LINE_JUNK(line)
print("TestJunkAPIs::test_is_line_junk_false: ok")
"###);
    assert_output(&out, r###"TestJunkAPIs::test_is_line_junk_false: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_junk_ap_is__test_is_line_junk_redos.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_junk_ap_is__test_is_line_junk_redos() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_junk_ap_is__test_is_line_junk_redos"
# subject = "cpython.test_difflib.TestJunkAPIs.test_is_line_junk_REDOS"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestJunkAPIs::test_is_line_junk_REDOS
"""Auto-ported test: TestJunkAPIs::test_is_line_junk_REDOS (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
evil_input = '\t' * 1000000 + '##'

assert not difflib.IS_LINE_JUNK(evil_input)
print("TestJunkAPIs::test_is_line_junk_REDOS: ok")
"###);
    assert_output(&out, r###"TestJunkAPIs::test_is_line_junk_REDOS: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_junk_ap_is__test_is_line_junk_true.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_junk_ap_is__test_is_line_junk_true() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_junk_ap_is__test_is_line_junk_true"
# subject = "cpython.test_difflib.TestJunkAPIs.test_is_line_junk_true"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestJunkAPIs::test_is_line_junk_true
"""Auto-ported test: TestJunkAPIs::test_is_line_junk_true (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
for line in ['#', '  ', ' #', '# ', ' # ', '']:

    assert difflib.IS_LINE_JUNK(line)
print("TestJunkAPIs::test_is_line_junk_true: ok")
"###);
    assert_output(&out, r###"TestJunkAPIs::test_is_line_junk_true: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_output_format__test_no_trailing_tab_on_empty_filedate.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_output_format__test_no_trailing_tab_on_empty_filedate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_output_format__test_no_trailing_tab_on_empty_filedate"
# subject = "cpython.test_difflib.TestOutputFormat.test_no_trailing_tab_on_empty_filedate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestOutputFormat::test_no_trailing_tab_on_empty_filedate
"""Auto-ported test: TestOutputFormat::test_no_trailing_tab_on_empty_filedate (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
args = ['one', 'two', 'Original', 'Current']
ud = difflib.unified_diff(*args, lineterm='')

assert list(ud)[0:2] == ['--- Original', '+++ Current']
cd = difflib.context_diff(*args, lineterm='')

assert list(cd)[0:2] == ['*** Original', '--- Current']
print("TestOutputFormat::test_no_trailing_tab_on_empty_filedate: ok")
"###);
    assert_output(&out, r###"TestOutputFormat::test_no_trailing_tab_on_empty_filedate: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_output_format__test_range_format_context.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_output_format__test_range_format_context() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_output_format__test_range_format_context"
# subject = "cpython.test_difflib.TestOutputFormat.test_range_format_context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestOutputFormat::test_range_format_context
"""Auto-ported test: TestOutputFormat::test_range_format_context (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
spec = '           The range of lines in file1 shall be written in the following format\n           if the range contains two or more lines:\n               "*** %d,%d ****\n", <beginning line number>, <ending line number>\n           and the following format otherwise:\n               "*** %d ****\n", <ending line number>\n           The ending line number of an empty range shall be the number of the preceding line,\n           or 0 if the range is at the start of the file.\n\n           Next, the range of lines in file2 shall be written in the following format\n           if the range contains two or more lines:\n               "--- %d,%d ----\n", <beginning line number>, <ending line number>\n           and the following format otherwise:\n               "--- %d ----\n", <ending line number>\n        '
fmt = difflib._format_range_context

assert fmt(3, 3) == '3'

assert fmt(3, 4) == '4'

assert fmt(3, 5) == '4,5'

assert fmt(3, 6) == '4,6'

assert fmt(0, 0) == '0'
print("TestOutputFormat::test_range_format_context: ok")
"###);
    assert_output(&out, r###"TestOutputFormat::test_range_format_context: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_output_format__test_range_format_unified.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_output_format__test_range_format_unified() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_output_format__test_range_format_unified"
# subject = "cpython.test_difflib.TestOutputFormat.test_range_format_unified"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestOutputFormat::test_range_format_unified
"""Auto-ported test: TestOutputFormat::test_range_format_unified (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
spec = '           Each <range> field shall be of the form:\n             %1d", <beginning line number>  if the range contains exactly one line,\n           and:\n            "%1d,%1d", <beginning line number>, <number of lines> otherwise.\n           If a range is empty, its beginning line number shall be the number of\n           the line just before the range, or 0 if the empty range starts the file.\n        '
fmt = difflib._format_range_unified

assert fmt(3, 3) == '3,0'

assert fmt(3, 4) == '4'

assert fmt(3, 5) == '4,2'

assert fmt(3, 6) == '4,3'

assert fmt(0, 0) == '0,0'
print("TestOutputFormat::test_range_format_unified: ok")
"###);
    assert_output(&out, r###"TestOutputFormat::test_range_format_unified: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_output_format__test_tab_delimiter.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_output_format__test_tab_delimiter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_output_format__test_tab_delimiter"
# subject = "cpython.test_difflib.TestOutputFormat.test_tab_delimiter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestOutputFormat::test_tab_delimiter
"""Auto-ported test: TestOutputFormat::test_tab_delimiter (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
args = ['one', 'two', 'Original', 'Current', '2005-01-26 23:30:50', '2010-04-02 10:20:52']
ud = difflib.unified_diff(*args, lineterm='')

assert list(ud)[0:2] == ['--- Original\t2005-01-26 23:30:50', '+++ Current\t2010-04-02 10:20:52']
cd = difflib.context_diff(*args, lineterm='')

assert list(cd)[0:2] == ['*** Original\t2005-01-26 23:30:50', '--- Current\t2010-04-02 10:20:52']
print("TestOutputFormat::test_tab_delimiter: ok")
"###);
    assert_output(&out, r###"TestOutputFormat::test_tab_delimiter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fbugs__test_added_tab_hint.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fbugs__test_added_tab_hint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fbugs__test_added_tab_hint"
# subject = "cpython.test_difflib.TestSFbugs.test_added_tab_hint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFbugs::test_added_tab_hint
"""Auto-ported test: TestSFbugs::test_added_tab_hint (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
diff = list(difflib.Differ().compare(['\tI am a buggy'], ['\t\tI am a bug']))

assert '- \tI am a buggy' == diff[0]

assert '? \t          --\n' == diff[1]

assert '+ \t\tI am a bug' == diff[2]

assert '? +\n' == diff[3]
print("TestSFbugs::test_added_tab_hint: ok")
"###);
    assert_output(&out, r###"TestSFbugs::test_added_tab_hint: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fbugs__test_comparing_empty_lists.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fbugs__test_comparing_empty_lists() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fbugs__test_comparing_empty_lists"
# subject = "cpython.test_difflib.TestSFbugs.test_comparing_empty_lists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFbugs::test_comparing_empty_lists
"""Auto-ported test: TestSFbugs::test_comparing_empty_lists (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
group_gen = difflib.SequenceMatcher(None, [], []).get_grouped_opcodes()

try:
    next(group_gen)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
diff_gen = difflib.unified_diff([], [])

try:
    next(diff_gen)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("TestSFbugs::test_comparing_empty_lists: ok")
"###);
    assert_output(&out, r###"TestSFbugs::test_comparing_empty_lists: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fbugs__test_hint_indented_properly_with_tabs.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fbugs__test_hint_indented_properly_with_tabs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fbugs__test_hint_indented_properly_with_tabs"
# subject = "cpython.test_difflib.TestSFbugs.test_hint_indented_properly_with_tabs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFbugs::test_hint_indented_properly_with_tabs
"""Auto-ported test: TestSFbugs::test_hint_indented_properly_with_tabs (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
diff = list(difflib.Differ().compare(['\t \t \t^'], ['\t \t \t^\n']))

assert '- \t \t \t^' == diff[0]

assert '+ \t \t \t^\n' == diff[1]

assert '? \t \t \t +\n' == diff[2]
print("TestSFbugs::test_hint_indented_properly_with_tabs: ok")
"###);
    assert_output(&out, r###"TestSFbugs::test_hint_indented_properly_with_tabs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fbugs__test_matching_blocks_cache.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fbugs__test_matching_blocks_cache() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fbugs__test_matching_blocks_cache"
# subject = "cpython.test_difflib.TestSFbugs.test_matching_blocks_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFbugs::test_matching_blocks_cache
"""Auto-ported test: TestSFbugs::test_matching_blocks_cache (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
s = difflib.SequenceMatcher(None, 'abxcd', 'abcd')
first = s.get_matching_blocks()
second = s.get_matching_blocks()

assert second[0].size == 2

assert second[1].size == 2

assert second[2].size == 0
print("TestSFbugs::test_matching_blocks_cache: ok")
"###);
    assert_output(&out, r###"TestSFbugs::test_matching_blocks_cache: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fbugs__test_mdiff_catch_stop_iteration.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fbugs__test_mdiff_catch_stop_iteration() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fbugs__test_mdiff_catch_stop_iteration"
# subject = "cpython.test_difflib.TestSFbugs.test_mdiff_catch_stop_iteration"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFbugs::test_mdiff_catch_stop_iteration
"""Auto-ported test: TestSFbugs::test_mdiff_catch_stop_iteration (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---

assert list(difflib._mdiff(['2'], ['3'], 1)) == [((1, '\x00-2\x01'), (1, '\x00+3\x01'), True)]
print("TestSFbugs::test_mdiff_catch_stop_iteration: ok")
"###);
    assert_output(&out, r###"TestSFbugs::test_mdiff_catch_stop_iteration: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fbugs__test_ratio_for_null_seqn.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fbugs__test_ratio_for_null_seqn() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fbugs__test_ratio_for_null_seqn"
# subject = "cpython.test_difflib.TestSFbugs.test_ratio_for_null_seqn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFbugs::test_ratio_for_null_seqn
"""Auto-ported test: TestSFbugs::test_ratio_for_null_seqn (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
s = difflib.SequenceMatcher(None, [], [])

assert s.ratio() == 1

assert s.quick_ratio() == 1

assert s.real_quick_ratio() == 1
print("TestSFbugs::test_ratio_for_null_seqn: ok")
"###);
    assert_output(&out, r###"TestSFbugs::test_ratio_for_null_seqn: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fpatches__test_make_file_default_charset.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fpatches__test_make_file_default_charset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fpatches__test_make_file_default_charset"
# subject = "cpython.test_difflib.TestSFpatches.test_make_file_default_charset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFpatches::test_make_file_default_charset
"""Auto-ported test: TestSFpatches::test_make_file_default_charset (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
html_diff = difflib.HtmlDiff()
output = html_diff.make_file(patch914575_from1.splitlines(), patch914575_to1.splitlines())

assert 'content="text/html; charset=utf-8"' in output
print("TestSFpatches::test_make_file_default_charset: ok")
"###);
    assert_output(&out, r###"TestSFpatches::test_make_file_default_charset: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fpatches__test_make_file_iso88591_charset.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fpatches__test_make_file_iso88591_charset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fpatches__test_make_file_iso88591_charset"
# subject = "cpython.test_difflib.TestSFpatches.test_make_file_iso88591_charset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFpatches::test_make_file_iso88591_charset
"""Auto-ported test: TestSFpatches::test_make_file_iso88591_charset (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
html_diff = difflib.HtmlDiff()
output = html_diff.make_file(patch914575_from1.splitlines(), patch914575_to1.splitlines(), charset='iso-8859-1')

assert 'content="text/html; charset=iso-8859-1"' in output
print("TestSFpatches::test_make_file_iso88591_charset: ok")
"###);
    assert_output(&out, r###"TestSFpatches::test_make_file_iso88591_charset: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fpatches__test_make_file_usascii_charset_with_nonascii_input.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fpatches__test_make_file_usascii_charset_with_nonascii_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fpatches__test_make_file_usascii_charset_with_nonascii_input"
# subject = "cpython.test_difflib.TestSFpatches.test_make_file_usascii_charset_with_nonascii_input"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFpatches::test_make_file_usascii_charset_with_nonascii_input
"""Auto-ported test: TestSFpatches::test_make_file_usascii_charset_with_nonascii_input (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
html_diff = difflib.HtmlDiff()
output = html_diff.make_file(patch914575_nonascii_from1.splitlines(), patch914575_nonascii_to1.splitlines(), charset='us-ascii')

assert 'content="text/html; charset=us-ascii"' in output

assert '&#305;mpl&#305;c&#305;t' in output
print("TestSFpatches::test_make_file_usascii_charset_with_nonascii_input: ok")
"###);
    assert_output(&out, r###"TestSFpatches::test_make_file_usascii_charset_with_nonascii_input: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/test_s_fpatches__test_recursion_limit.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_test_s_fpatches__test_recursion_limit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fpatches__test_recursion_limit"
# subject = "cpython.test_difflib.TestSFpatches.test_recursion_limit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_difflib.py::TestSFpatches::test_recursion_limit
"""Auto-ported test: TestSFpatches::test_recursion_limit (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
limit = sys.getrecursionlimit()
old = [(i % 2 and 'K:%d' or 'V:A:%d') % i for i in range(limit * 2)]
new = [(i % 2 and 'K:%d' or 'V:B:%d') % i for i in range(limit * 2)]
difflib.SequenceMatcher(None, old, new).get_opcodes()
print("TestSFpatches::test_recursion_limit: ok")
"###);
    assert_output(&out, r###"TestSFpatches::test_recursion_limit: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/unified_diff_headers_and_lines.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_unified_diff_headers_and_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "unified_diff_headers_and_lines"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff emits ---/+++ file headers and the -<removed>/+<added> body lines for a single changed line"""
import difflib

_old = "line1\nline2\nline3\n".splitlines(keepends=True)
_new = "line1\nchanged\nline3\n".splitlines(keepends=True)
_ud = list(difflib.unified_diff(
    _old, _new, fromfile="old", tofile="new", lineterm=""))
assert any(line.startswith("---") for line in _ud), f"--- header missing: {_ud!r}"
assert any(line.startswith("+++") for line in _ud), f"+++ header missing: {_ud!r}"
assert any(line.startswith("-line2") for line in _ud), f"-line2 missing: {_ud!r}"
assert any(line.startswith("+changed") for line in _ud), f"+changed missing: {_ud!r}"
print("unified_diff_headers_and_lines OK")
"###);
    assert_output(&out, r###"unified_diff_headers_and_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/unified_diff_hunk_header.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_unified_diff_hunk_header() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "unified_diff_hunk_header"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff emits an '@@ -1,3 +1,3 @@' hunk header for a 3-line file with one replaced line"""
import difflib

_a = "a\nb\nc\n".splitlines(keepends=True)
_b = "a\nX\nc\n".splitlines(keepends=True)
_ud = list(difflib.unified_diff(_a, _b, lineterm=""))
assert "@@ -1,3 +1,3 @@" in _ud, f"unified hunk header = {_ud!r}"
print("unified_diff_hunk_header OK")
"###);
    assert_output(&out, r###"unified_diff_hunk_header OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/difflib/unified_diff_tab_delimited_filedates.py`.
#[test]
fn test_gen_behavior_std_libs_difflib_unified_diff_tab_delimited_filedates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "unified_diff_tab_delimited_filedates"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.unified_diff: with filedates the ---/+++ headers are tab-separated 'name\\tdate'; without filedates there is no trailing tab"""
import difflib

_hdr = list(difflib.unified_diff(
    ["one"], ["two"], "Original", "Current",
    "2005-01-26 23:30:50", "2010-04-02 10:20:52", lineterm=""))
assert _hdr[0] == "--- Original\t2005-01-26 23:30:50", f"from header = {_hdr[0]!r}"
assert _hdr[1] == "+++ Current\t2010-04-02 10:20:52", f"to header = {_hdr[1]!r}"
# Without filedates there is no trailing tab.
_hdr2 = list(difflib.unified_diff(
    ["one"], ["two"], "Original", "Current", lineterm=""))
assert _hdr2[0] == "--- Original", f"bare from header = {_hdr2[0]!r}"
assert _hdr2[1] == "+++ Current", f"bare to header = {_hdr2[1]!r}"
print("unified_diff_tab_delimited_filedates OK")
"###);
    assert_output(&out, r###"unified_diff_tab_delimited_filedates OK
"###);
}
