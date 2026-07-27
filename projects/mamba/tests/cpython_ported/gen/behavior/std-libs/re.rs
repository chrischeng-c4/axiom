use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/re/atomic_group_no_backtrack.py`.
#[test]
fn test_gen_behavior_std_libs_re_atomic_group_no_backtrack() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "atomic_group_no_backtrack"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: an atomic group (?>bc|b) takes its match and refuses to give it back: r'a(?>bc|b)c' rejects 'abc' but accepts 'abcc'"""
import re

pat = re.compile(r"a(?>bc|b)c")
assert pat.match("abc") is None, "atomic refuses to backtrack to b"
assert pat.match("abcc") is not None, "atomic matches bc then c"
assert re.match(r"(?>.*).", "abc") is None, "atomic .* leaves nothing"

print("atomic_group_no_backtrack OK")
"###);
    assert_output(&out, r###"atomic_group_no_backtrack OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/class_shortcut_escapes.py`.
#[test]
fn test_gen_behavior_std_libs_re_class_shortcut_escapes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "class_shortcut_escapes"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.search: the class shortcuts \\d \\D \\w \\W \\s \\S match digit/non-digit, word/non-word, space/non-space respectively, for both str and bytes patterns"""
import re

assert re.search(r"\d\D\w\W\s\S", "1aa! a").group(0) == "1aa! a", "class shortcuts str"
assert re.search(rb"\d\D\w\W\s\S", b"1aa! a").group(0) == b"1aa! a", "class shortcuts bytes"

print("class_shortcut_escapes OK")
"###);
    assert_output(&out, r###"class_shortcut_escapes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/compiled_pattern_reuse.py`.
#[test]
fn test_gen_behavior_std_libs_re_compiled_pattern_reuse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "compiled_pattern_reuse"
# subject = "re.Pattern.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.findall: a compiled pattern is reusable across method calls: re.compile(r'[aeiou]').findall('hello world') is ['e','o','o'] and .sub('_','hello') is 'h_ll_'"""
import re

pat = re.compile(r"[aeiou]")
assert pat.findall("hello world") == ["e", "o", "o"], "compiled findall"
assert pat.sub("_", "hello") == "h_ll_", "compiled sub reuse"

print("compiled_pattern_reuse OK")
"###);
    assert_output(&out, r###"compiled_pattern_reuse OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/conditional_group.py`.
#[test]
fn test_gen_behavior_std_libs_re_conditional_group() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "conditional_group"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: a conditional group (?(1)yes|no) branches on whether group 1 matched: r'^(\\()?([^()]+)(?(1)\\))$' accepts '(a)' and 'a' but rejects 'a)' and '(a'"""
import re

pat = r"^(\()?([^()]+)(?(1)\))$"
assert re.match(pat, "(a)").groups() == ("(", "a"), "conditional with paren"
assert re.match(pat, "a").groups() == (None, "a"), "conditional no paren"
assert re.match(pat, "a)") is None, "conditional rejects stray )"
assert re.match(pat, "(a") is None, "conditional rejects missing )"

print("conditional_group OK")
"###);
    assert_output(&out, r###"conditional_group OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/dotall_flag.py`.
#[test]
fn test_gen_behavior_std_libs_re_dotall_flag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "dotall_flag"
# subject = "re.DOTALL"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.DOTALL: by default '.' does not match newline; re.DOTALL makes 'a.b' match 'a\\nb'"""
import re

assert re.search(r"a.b", "a\nb") is None, "dot does not match newline by default"
assert re.search(r"a.b", "a\nb", re.DOTALL) is not None, "DOTALL -> dot matches newline"

print("dotall_flag OK")
"###);
    assert_output(&out, r###"dotall_flag OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/escape_makes_metachars_literal.py`.
#[test]
fn test_gen_behavior_std_libs_re_escape_makes_metachars_literal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "escape_makes_metachars_literal"
# subject = "re.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.escape: re.escape turns metacharacters into literals: the escaped 'a.b+c?' matches that exact text but the escaped dot does NOT match an arbitrary char"""
import re

escaped = re.escape("a.b+c?")
assert re.search(escaped, "a.b+c?") is not None, "escaped text matches itself"
assert re.search(escaped, "axb+c?") is None, "escaped dot does NOT match any char"

print("escape_makes_metachars_literal OK")
"###);
    assert_output(&out, r###"escape_makes_metachars_literal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/findall_alternation.py`.
#[test]
fn test_gen_behavior_std_libs_re_findall_alternation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_alternation"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.findall: findall over an alternation collects each alternative hit: r'cat|dog' on 'I have a cat and a dog' is ['cat','dog']"""
import re

assert re.findall(r"cat|dog", "I have a cat and a dog") == ["cat", "dog"]

print("findall_alternation OK")
"###);
    assert_output(&out, r###"findall_alternation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/findall_multi_groups_returns_tuples.py`.
#[test]
fn test_gen_behavior_std_libs_re_findall_multi_groups_returns_tuples() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_multi_groups_returns_tuples"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.findall: findall with multiple groups returns a list of group tuples: r'(\\w+)=(\\d+)' on 'a=1 b=2 c=3' is [('a','1'),('b','2'),('c','3')]"""
import re

matches = re.findall(r"(\w+)=(\d+)", "a=1 b=2 c=3")
assert matches == [("a", "1"), ("b", "2"), ("c", "3")], f"findall groups = {matches!r}"

print("findall_multi_groups_returns_tuples OK")
"###);
    assert_output(&out, r###"findall_multi_groups_returns_tuples OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/findall_no_groups_returns_flat_list.py`.
#[test]
fn test_gen_behavior_std_libs_re_findall_no_groups_returns_flat_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_no_groups_returns_flat_list"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.findall: findall with no capture groups returns a flat list of whole matches: r'\\w+' on 'hello world foo' is ['hello','world','foo']"""
import re

assert re.findall(r"\w+", "hello world foo") == ["hello", "world", "foo"]

print("findall_no_groups_returns_flat_list OK")
"###);
    assert_output(&out, r###"findall_no_groups_returns_flat_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/findall_no_match_returns_empty_list.py`.
#[test]
fn test_gen_behavior_std_libs_re_findall_no_match_returns_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_no_match_returns_empty_list"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.findall: findall returns an empty list (never None) when nothing matches: r'\\d+' on 'no digits' and on '' are both []"""
import re

assert re.findall(r"\d+", "no digits") == []
assert re.findall(r"\d+", "") == []

print("findall_no_match_returns_empty_list OK")
"###);
    assert_output(&out, r###"findall_no_match_returns_empty_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/findall_one_group_returns_group_text.py`.
#[test]
fn test_gen_behavior_std_libs_re_findall_one_group_returns_group_text() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_one_group_returns_group_text"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.findall: findall with one capture group returns the group's text, not the whole match: r'(\\d+)' on 'abc123def456' is ['123','456']"""
import re

assert re.findall(r"(\d+)", "abc123def456") == ["123", "456"]

print("findall_one_group_returns_group_text OK")
"###);
    assert_output(&out, r###"findall_one_group_returns_group_text OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/finditer_yields_matches.py`.
#[test]
fn test_gen_behavior_std_libs_re_finditer_yields_matches() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "finditer_yields_matches"
# subject = "re.Pattern.finditer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Pattern.finditer: finditer yields Match objects in order, and honors the optional pos/endpos window: re.compile(r':+').finditer over 'a:b::c:::d' all vs windowed (3,8)"""
import re

fp = re.compile(r":+")
assert [x.group() for x in fp.finditer("a:b::c:::d")] == [":", "::", ":::"], "finditer all"
assert [x.group() for x in fp.finditer("a:b::c:::d", 3, 8)] == ["::", "::"], "finditer window"

print("finditer_yields_matches OK")
"###);
    assert_output(&out, r###"finditer_yields_matches OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/fullmatch_requires_whole_string.py`.
#[test]
fn test_gen_behavior_std_libs_re_fullmatch_requires_whole_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "fullmatch_requires_whole_string"
# subject = "re.fullmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.fullmatch: re.fullmatch requires the whole string to match: r'\\d{3}' matches '123' but returns None for '1234'"""
import re

assert re.fullmatch(r"\d{3}", "123") is not None, "fullmatch exact"
assert re.fullmatch(r"\d{3}", "1234") is None, "fullmatch too long -> None"

print("fullmatch_requires_whole_string OK")
"###);
    assert_output(&out, r###"fullmatch_requires_whole_string OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/getitem_index_and_name.py`.
#[test]
fn test_gen_behavior_std_libs_re_getitem_index_and_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "getitem_index_and_name"
# subject = "re.Match.__getitem__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.__getitem__: Match supports m[index] and m[name]; unmatched optional/alternation groups give None"""
import re

pat = re.compile(r"(?:(?P<a1>a)|(?P<b2>b))(?P<c3>c)?")
m = pat.match("a")
assert m[0] == "a", f"m[0] = {m[0]!r}"
assert m[1] == "a", f"m[1] = {m[1]!r}"
assert m[2] is None, "unmatched alternation group -> None"
assert m["a1"] == "a", "getitem by name"
assert m["b2"] is None, "unmatched named group -> None"
assert m["c3"] is None, "unmatched optional -> None"

print("getitem_index_and_name OK")
"###);
    assert_output(&out, r###"getitem_index_and_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/group_multi_id_returns_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_re_group_multi_id_returns_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "group_multi_id_returns_tuple"
# subject = "re.Match.group"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.group: group() with several ids returns them in id order: re.match(r'(a)(b)','ab').group(2,1) is ('b','a'), and group()==group(0)=='ab'"""
import re

m = re.match(r"(a)(b)", "ab")
assert m.group(2, 1) == ("b", "a"), "multi-id tuple in id order"
assert m.group() == "ab" and m.group(0) == "ab", "group() == group(0)"

print("group_multi_id_returns_tuple OK")
"###);
    assert_output(&out, r###"group_multi_id_returns_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/ignorecase_flag.py`.
#[test]
fn test_gen_behavior_std_libs_re_ignorecase_flag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "ignorecase_flag"
# subject = "re.IGNORECASE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.IGNORECASE: re.IGNORECASE makes the match case-insensitive: re.search(r'hello','HELLO',re.IGNORECASE) is not None"""
import re

assert re.search(r"hello", "HELLO", re.IGNORECASE) is not None, "IGNORECASE match"
assert re.search(r"hello", "HELLO") is None, "no flag -> no match"

print("ignorecase_flag OK")
"###);
    assert_output(&out, r###"ignorecase_flag OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/inline_scoped_flags.py`.
#[test]
fn test_gen_behavior_std_libs_re_inline_scoped_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "inline_scoped_flags"
# subject = "re.fullmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.fullmatch: (?i) enables case-insensitivity globally while scoped (?i:...) limits it to that group and (?-x:...) re-enables whitespace significance inside"""
import re

# (?i) global inline ignorecase.
assert re.match(r"(?i)abc", "ABC").group(0) == "ABC", "(?i) inline ignorecase"
# Scoped (?i:...) limits the flag to that group.
assert re.fullmatch(r"a(?i:b)c", "aBc") is not None, "scoped (?i:) matches B"
assert re.fullmatch(r"a(?i:b)c", "AbC") is None, "scoped flag does not leak out"
# Scoped disable (?-x:...) re-enables whitespace significance inside.
assert re.fullmatch(r"(?x) a(?-x: b) c", "a bc") is not None, "scoped (?-x:) keeps space"

print("inline_scoped_flags OK")
"###);
    assert_output(&out, r###"inline_scoped_flags OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/match_anchors_at_start.py`.
#[test]
fn test_gen_behavior_std_libs_re_match_anchors_at_start() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_anchors_at_start"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.match: re.match anchors at the start: r'\\d+' matches '123abc' (group '123') but returns None for 'abc123'"""
import re

m = re.match(r"\d+", "123abc")
assert m is not None, "match at start"
assert m.group() == "123", f"group = {m.group()!r}"
assert re.match(r"\d+", "abc123") is None, "no match when not at start"

print("match_anchors_at_start OK")
"###);
    assert_output(&out, r###"match_anchors_at_start OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/match_empty_pattern_succeeds.py`.
#[test]
fn test_gen_behavior_std_libs_re_match_empty_pattern_succeeds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_empty_pattern_succeeds"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.match: the empty pattern matches at the start of any string: re.match('', 'abc') is not None"""
import re

m = re.match(r"", "abc")
assert m is not None, "empty pattern matches at start"
assert m.group() == "", f"empty match group = {m.group()!r}"

print("match_empty_pattern_succeeds OK")
"###);
    assert_output(&out, r###"match_empty_pattern_succeeds OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/match_expand_template.py`.
#[test]
fn test_gen_behavior_std_libs_re_match_expand_template() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_expand_template"
# subject = "re.Match.expand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.expand: Match.expand fills a template from the captured groups by number (\\1) and by name (\\g<name>)"""
import re

m = re.match(r"(?P<first>\w+) (?P<second>\w+)", "hello world")
assert m.expand(r"\2 \1") == "world hello", "expand by number"
assert m.expand(r"\g<second>-\g<first>") == "world-hello", "expand by name"

print("match_expand_template OK")
"###);
    assert_output(&out, r###"match_expand_template OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/match_format_map.py`.
#[test]
fn test_gen_behavior_std_libs_re_match_format_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_format_map"
# subject = "re.Match.__getitem__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.__getitem__: a Match object is usable as the mapping in str.format_map; unmatched named groups render as 'None'"""
import re

pat = re.compile(r"(?:(?P<a1>a)|(?P<b2>b))(?P<c3>c)?")
m = pat.match("a")
assert "{a1}/{b2}/{c3}".format_map(m) == "a/None/None", "format_map over match"

print("match_format_map OK")
"###);
    assert_output(&out, r###"match_format_map OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/match_pos_endpos_string_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_re_match_pos_endpos_string_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_pos_endpos_string_attrs"
# subject = "re.Match.string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.string: a Match exposes pos/endpos describing the search window and .string the original subject: re.match(r'(a)','a') has pos 0, endpos 1, string 'a'"""
import re

m = re.match(r"(a)", "a")
assert m.pos == 0 and m.endpos == 1, "pos/endpos describe the window"
assert m.string == "a", "match.string is the subject"
assert m.re is not None, "match.re present"

print("match_pos_endpos_string_attrs OK")
"###);
    assert_output(&out, r###"match_pos_endpos_string_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/multiline_flag.py`.
#[test]
fn test_gen_behavior_std_libs_re_multiline_flag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "multiline_flag"
# subject = "re.MULTILINE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.MULTILINE: re.MULTILINE makes ^ and $ per-line: r'^\\w+' over 'line1\\nline2\\nline3' findall is ['line1','line2','line3']"""
import re

text = "line1\nline2\nline3"
assert re.findall(r"^\w+", text, re.MULTILINE) == ["line1", "line2", "line3"]

print("multiline_flag OK")
"###);
    assert_output(&out, r###"multiline_flag OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/named_groups_and_groupdict.py`.
#[test]
fn test_gen_behavior_std_libs_re_named_groups_and_groupdict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "named_groups_and_groupdict"
# subject = "re.Match.groupdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.groupdict: named groups resolve by name and groupdict() returns the name->text map: r'(?P<year>\\d{4})-(?P<month>\\d{2})' on '2024-03'"""
import re

m = re.search(r"(?P<year>\d{4})-(?P<month>\d{2})", "date: 2024-03")
assert m is not None, "named groups match"
assert m.group("year") == "2024", f"year = {m.group('year')!r}"
assert m.group("month") == "03", f"month = {m.group('month')!r}"
assert m.groupdict() == {"year": "2024", "month": "03"}, f"groupdict = {m.groupdict()!r}"

print("named_groups_and_groupdict OK")
"###);
    assert_output(&out, r###"named_groups_and_groupdict OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/numbered_groups.py`.
#[test]
fn test_gen_behavior_std_libs_re_numbered_groups() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "numbered_groups"
# subject = "re.Match.group"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.group: numbered groups: re.search(r'(\\w+)\\s+(\\w+)','hello world') gives group(1)='hello', group(2)='world', groups()=('hello','world')"""
import re

m = re.search(r"(\w+)\s+(\w+)", "hello world")
assert m is not None, "groups match"
assert m.group(1) == "hello", f"g1 = {m.group(1)!r}"
assert m.group(2) == "world", f"g2 = {m.group(2)!r}"
assert m.groups() == ("hello", "world"), f"groups() = {m.groups()!r}"

print("numbered_groups OK")
"###);
    assert_output(&out, r###"numbered_groups OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/pattern_groups_groupindex_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_re_pattern_groups_groupindex_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "pattern_groups_groupindex_attrs"
# subject = "re.Pattern.groupindex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.groupindex: a compiled Pattern reports .groups count, .groupindex name->number map, and the original .pattern source string"""
import re

cp = re.compile(r"(?P<first>a)(?P<other>b)", re.I)
assert cp.groups == 2, "pattern.groups count"
assert cp.groupindex == {"first": 1, "other": 2}, "pattern.groupindex map"
assert cp.pattern == r"(?P<first>a)(?P<other>b)", "pattern.pattern source"

print("pattern_groups_groupindex_attrs OK")
"###);
    assert_output(&out, r###"pattern_groups_groupindex_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/pattern_repr_tests__test_quotes.py`.
#[test]
fn test_gen_behavior_std_libs_re_pattern_repr_tests__test_quotes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "pattern_repr_tests__test_quotes"
# subject = "cpython.test_re.PatternReprTests.test_quotes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::PatternReprTests::test_quotes
"""Auto-ported test: PatternReprTests::test_quotes (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
def check(pattern, expected):

    assert repr(re.compile(pattern)) == expected

def check_flags(pattern, flags, expected):

    assert repr(re.compile(pattern, flags)) == expected
check('random "double quoted" pattern', 're.compile(\'random "double quoted" pattern\')')
check("random 'single quoted' pattern", 're.compile("random \'single quoted\' pattern")')
check('both \'single\' and "double" quotes', 're.compile(\'both \\\'single\\\' and "double" quotes\')')
print("PatternReprTests::test_quotes: ok")
"###);
    assert_output(&out, r###"PatternReprTests::test_quotes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/pattern_repr_tests__test_without_flags.py`.
#[test]
fn test_gen_behavior_std_libs_re_pattern_repr_tests__test_without_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "pattern_repr_tests__test_without_flags"
# subject = "cpython.test_re.PatternReprTests.test_without_flags"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::PatternReprTests::test_without_flags
"""Auto-ported test: PatternReprTests::test_without_flags (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
def check(pattern, expected):

    assert repr(re.compile(pattern)) == expected

def check_flags(pattern, flags, expected):

    assert repr(re.compile(pattern, flags)) == expected
check('random pattern', "re.compile('random pattern')")
print("PatternReprTests::test_without_flags: ok")
"###);
    assert_output(&out, r###"PatternReprTests::test_without_flags: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/positive_negative_lookahead.py`.
#[test]
fn test_gen_behavior_std_libs_re_positive_negative_lookahead() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "positive_negative_lookahead"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: positive lookahead (?=...) constrains without consuming (r'a(?=\\d)' on 'a5' matches 'a', end 1); negative lookahead (?!...) blocks the forbidden follow"""
import re

# Positive lookahead: 'a' followed by a digit, the digit is not consumed.
m = re.match(r"a(?=\d)", "a5")
assert m is not None and m.group() == "a", f"lookahead group = {m.group()!r}"
assert m.end() == 1, f"lookahead end = {m.end()!r}"
# Negative lookahead: 'a' NOT followed by a digit.
assert re.match(r"a(?!\d)", "ab") is not None, "neg lookahead ok"
assert re.match(r"a(?!\d)", "a5") is None, "neg lookahead blocks digit"

print("positive_negative_lookahead OK")
"###);
    assert_output(&out, r###"positive_negative_lookahead OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/positive_negative_lookbehind.py`.
#[test]
fn test_gen_behavior_std_libs_re_positive_negative_lookbehind() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "positive_negative_lookbehind"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.search: positive lookbehind (?<=...) requires the preceding text (r'(?<=b)c' matches 'abc'); negative lookbehind (?<!...) forbids it"""
import re

assert re.search(r"(?<=b)c", "abc") is not None, "lookbehind ok"
assert re.search(r"(?<=x)c", "abc") is None, "lookbehind wrong prefix"
assert re.match(r"ab(?<!c)c", "abc") is not None, "neg lookbehind ok"
assert re.match(r"ab(?<=c)c", "abc") is None, "lookbehind needs c before"

print("positive_negative_lookbehind OK")
"###);
    assert_output(&out, r###"positive_negative_lookbehind OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/possessive_quantifiers.py`.
#[test]
fn test_gen_behavior_std_libs_re_possessive_quantifiers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "possessive_quantifiers"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: possessive quantifiers (*+, ++, ?+, {m,n}+) never give back: r'e*+e' fails on 'eeee' while r'e++a' matches 'eeea'"""
import re

assert re.match(r"e*+e", "eeee") is None, "e*+ eats all, no e left"
assert re.match(r"e++a", "eeea").group(0) == "eeea", "e++a"
assert re.match(r"e?+a", "ea").group(0) == "ea", "e?+a"
assert re.match(r"e{2,4}+a", "eeea").group(0) == "eeea", "e{2,4}+a"
assert re.search(r"x++", "axx").span() == (1, 3), "x++ span"

print("possessive_quantifiers OK")
"###);
    assert_output(&out, r###"possessive_quantifiers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_big_codesize.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_big_codesize() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_big_codesize"
# subject = "cpython.test_re.ReTests.test_big_codesize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_big_codesize
"""Auto-ported test: ReTests::test_big_codesize (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
r = re.compile('|'.join(('%d' % x for x in range(10000))))

assert r.match('1000')

assert r.match('9999')
print("ReTests::test_big_codesize: ok")
"###);
    assert_output(&out, r###"ReTests::test_big_codesize: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bigcharset.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bigcharset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bigcharset"
# subject = "cpython.test_re.ReTests.test_bigcharset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bigcharset
"""Auto-ported test: ReTests::test_bigcharset (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('([∢∣])', '∢').group(1) == '∢'
r = '[%s]' % ''.join(map(chr, range(256, 2 ** 16, 255)))

assert re.match(r, '！').group() == '！'
print("ReTests::test_bigcharset: ok")
"###);
    assert_output(&out, r###"ReTests::test_bigcharset: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_branching.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_branching() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_branching"
# subject = "cpython.test_re.ReTests.test_branching"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_branching
"""Auto-ported test: ReTests::test_branching (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
"Test Branching\n        Test expressions using the OR ('|') operator."

assert re.match('(ab|ba)', 'ab').span() == (0, 2)

assert re.match('(ab|ba)', 'ba').span() == (0, 2)

assert re.match('(abc|bac|ca|cb)', 'abc').span() == (0, 3)

assert re.match('(abc|bac|ca|cb)', 'bac').span() == (0, 3)

assert re.match('(abc|bac|ca|cb)', 'ca').span() == (0, 2)

assert re.match('(abc|bac|ca|cb)', 'cb').span() == (0, 2)

assert re.match('((a)|(b)|(c))', 'a').span() == (0, 1)

assert re.match('((a)|(b)|(c))', 'b').span() == (0, 1)

assert re.match('((a)|(b)|(c))', 'c').span() == (0, 1)
print("ReTests::test_branching: ok")
"###);
    assert_output(&out, r###"ReTests::test_branching: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_113254.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_113254() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_113254"
# subject = "cpython.test_re.ReTests.test_bug_113254"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_113254
"""Auto-ported test: ReTests::test_bug_113254 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(a)|(b)', 'b').start(1) == -1

assert re.match('(a)|(b)', 'b').end(1) == -1

assert re.match('(a)|(b)', 'b').span(1) == (-1, -1)
print("ReTests::test_bug_113254: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_113254: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_117612.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_117612() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_117612"
# subject = "cpython.test_re.ReTests.test_bug_117612"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_117612
"""Auto-ported test: ReTests::test_bug_117612 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.findall('(a|(b))', 'aba') == [('a', ''), ('b', 'b'), ('a', '')]
print("ReTests::test_bug_117612: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_117612: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_448951.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_448951() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_448951"
# subject = "cpython.test_re.ReTests.test_bug_448951"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_448951
"""Auto-ported test: ReTests::test_bug_448951 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
for op in ('', '?', '*'):

    assert re.match('((.%s):)?z' % op, 'z').groups() == (None, None)

    assert re.match('((.%s):)?z' % op, 'a:z').groups() == ('a:', 'a')
print("ReTests::test_bug_448951: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_448951: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_612074.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_612074() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_612074"
# subject = "cpython.test_re.ReTests.test_bug_612074"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_612074
"""Auto-ported test: ReTests::test_bug_612074 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
pat = '[' + re.escape('‹') + ']'

assert (re.compile(pat) and 1) == 1
print("ReTests::test_bug_612074: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_612074: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_6561.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_6561() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_6561"
# subject = "cpython.test_re.ReTests.test_bug_6561"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_6561
"""Auto-ported test: ReTests::test_bug_6561 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
decimal_digits = ['7', '๘', '０']
for x in decimal_digits:

    assert re.match('^\\d$', x).group(0) == x
not_decimal_digits = ['Ⅵ', '〹', '₂', '㊴']
for x in not_decimal_digits:

    assert re.match('^\\d$', x) is None
print("ReTests::test_bug_6561: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_6561: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_725106.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_725106() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_725106"
# subject = "cpython.test_re.ReTests.test_bug_725106"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_725106
"""Auto-ported test: ReTests::test_bug_725106 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('^((a)|b)*', 'abc').groups() == ('b', 'a')

assert re.match('^(([ab])|c)*', 'abc').groups() == ('c', 'b')

assert re.match('^((d)|[ab])*', 'abc').groups() == ('b', None)

assert re.match('^((a)c|[ab])*', 'abc').groups() == ('b', None)

assert re.match('^((a)|b)*?c', 'abc').groups() == ('b', 'a')

assert re.match('^(([ab])|c)*?d', 'abcd').groups() == ('c', 'b')

assert re.match('^((d)|[ab])*?c', 'abc').groups() == ('b', None)

assert re.match('^((a)c|[ab])*?c', 'abc').groups() == ('b', None)
print("ReTests::test_bug_725106: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_725106: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_926075.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_926075() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_926075"
# subject = "cpython.test_re.ReTests.test_bug_926075"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_926075
"""Auto-ported test: ReTests::test_bug_926075 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.compile('bug_926075') is not re.compile(b'bug_926075')
print("ReTests::test_bug_926075: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_926075: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_931848.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_931848() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_931848"
# subject = "cpython.test_re.ReTests.test_bug_931848"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_931848
"""Auto-ported test: ReTests::test_bug_931848 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
pattern = '[.。．｡]'

assert re.compile(pattern).split('a.b.c') == ['a', 'b', 'c']
print("ReTests::test_bug_931848: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_931848: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_bug_gh101955.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_bug_gh101955() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_gh101955"
# subject = "cpython.test_re.ReTests.test_bug_gh101955"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_gh101955
"""Auto-ported test: ReTests::test_bug_gh101955 (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('((x)|y|z)*+', 'xyz').groups() == ('z', 'x')

assert re.match('((x)|y|z){3}+', 'xyz').groups() == ('z', 'x')

assert re.match('((x)|y|z){3,}+', 'xyz').groups() == ('z', 'x')
print("ReTests::test_bug_gh101955: ok")
"###);
    assert_output(&out, r###"ReTests::test_bug_gh101955: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_category.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_category() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_category"
# subject = "cpython.test_re.ReTests.test_category"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_category
"""Auto-ported test: ReTests::test_category (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(\\s)', ' ').group(1) == ' '
print("ReTests::test_category: ok")
"###);
    assert_output(&out, r###"ReTests::test_category: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_character_set_any.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_character_set_any() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_character_set_any"
# subject = "cpython.test_re.ReTests.test_character_set_any"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_character_set_any
"""Auto-ported test: ReTests::test_character_set_any (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

def assertMatch(pattern, text, match=None, span=None, matcher=re.fullmatch):
    if match is None and span is None:
        match = text
        span = (0, len(text))
    elif match is None or span is None:
        raise ValueError('If match is not None, span should be specified (and vice versa).')
    m = matcher(pattern, text)

    assert m

    assert m.group() == match

    assert m.span() == span

def assertTypedEqual(actual, expect, msg=None):

    assert actual == expect

    def recurse(actual, expect):
        if isinstance(expect, (tuple, list)):
            for x, y in zip(actual, expect):
                recurse(x, y)
        else:
            self.assertIs(type(actual), type(expect), msg)
    recurse(actual, expect)

def bump_num(matchobj):
    int_value = int(matchobj.group(0))
    return str(int_value + 1)

def checkPatternError(pattern, errmsg, pos=None):
    try:
        re.compile(pattern)
        raise AssertionError('expected re.error')
    except re.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    err = cm.exception

    assert err.msg == errmsg
    if pos is not None:

        assert err.pos == pos

def checkTemplateError(pattern, repl, string, errmsg, pos=None):
    try:
        re.sub(pattern, repl, string)
        raise AssertionError('expected re.error')
    except re.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    err = cm.exception

    assert err.msg == errmsg
    if pos is not None:

        assert err.pos == pos

def check_en_US_iso88591():
    locale.setlocale(locale.LC_CTYPE, 'en_US.iso88591')

    assert re.match(b'\xc5\xe5', b'\xc5\xe5', re.L | re.I)

    assert re.match(b'\xc5', b'\xe5', re.L | re.I)

    assert re.match(b'\xe5', b'\xc5', re.L | re.I)

    assert re.match(b'(?Li)\xc5\xe5', b'\xc5\xe5')

    assert re.match(b'(?Li)\xc5', b'\xe5')

    assert re.match(b'(?Li)\xe5', b'\xc5')

def check_en_US_utf8():
    locale.setlocale(locale.LC_CTYPE, 'en_US.utf8')

    assert re.match(b'\xc5\xe5', b'\xc5\xe5', re.L | re.I)

    assert re.match(b'\xc5', b'\xe5', re.L | re.I) is None

    assert re.match(b'\xe5', b'\xc5', re.L | re.I) is None

    assert re.match(b'(?Li)\xc5\xe5', b'\xc5\xe5')

    assert re.match(b'(?Li)\xc5', b'\xe5') is None

    assert re.match(b'(?Li)\xe5', b'\xc5') is None

def check_interrupt(pattern, string, maxcount):

    class Interrupt(Exception):
        pass
    p = re.compile(pattern)
    for n in range(maxcount):
        try:
            p._fail_after(n, Interrupt)
            p.match(string)
            return n
        except Interrupt:
            pass
        finally:
            p._fail_after(-1, None)
s = '1x\n'
for p in ('[\\s\\S]', '[\\d\\D]', '[\\w\\W]', '[\\S\\s]', '\\s|\\S'):

    assert re.findall(p, s) == list(s)

    assert re.fullmatch('(?:' + p + ')+', s).group() == s
print("ReTests::test_character_set_any: ok")
"###);
    assert_output(&out, r###"ReTests::test_character_set_any: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_character_set_none.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_character_set_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_character_set_none"
# subject = "cpython.test_re.ReTests.test_character_set_none"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_character_set_none
"""Auto-ported test: ReTests::test_character_set_none (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

def assertMatch(pattern, text, match=None, span=None, matcher=re.fullmatch):
    if match is None and span is None:
        match = text
        span = (0, len(text))
    elif match is None or span is None:
        raise ValueError('If match is not None, span should be specified (and vice versa).')
    m = matcher(pattern, text)

    assert m

    assert m.group() == match

    assert m.span() == span

def assertTypedEqual(actual, expect, msg=None):

    assert actual == expect

    def recurse(actual, expect):
        if isinstance(expect, (tuple, list)):
            for x, y in zip(actual, expect):
                recurse(x, y)
        else:
            self.assertIs(type(actual), type(expect), msg)
    recurse(actual, expect)

def bump_num(matchobj):
    int_value = int(matchobj.group(0))
    return str(int_value + 1)

def checkPatternError(pattern, errmsg, pos=None):
    try:
        re.compile(pattern)
        raise AssertionError('expected re.error')
    except re.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    err = cm.exception

    assert err.msg == errmsg
    if pos is not None:

        assert err.pos == pos

def checkTemplateError(pattern, repl, string, errmsg, pos=None):
    try:
        re.sub(pattern, repl, string)
        raise AssertionError('expected re.error')
    except re.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    err = cm.exception

    assert err.msg == errmsg
    if pos is not None:

        assert err.pos == pos

def check_en_US_iso88591():
    locale.setlocale(locale.LC_CTYPE, 'en_US.iso88591')

    assert re.match(b'\xc5\xe5', b'\xc5\xe5', re.L | re.I)

    assert re.match(b'\xc5', b'\xe5', re.L | re.I)

    assert re.match(b'\xe5', b'\xc5', re.L | re.I)

    assert re.match(b'(?Li)\xc5\xe5', b'\xc5\xe5')

    assert re.match(b'(?Li)\xc5', b'\xe5')

    assert re.match(b'(?Li)\xe5', b'\xc5')

def check_en_US_utf8():
    locale.setlocale(locale.LC_CTYPE, 'en_US.utf8')

    assert re.match(b'\xc5\xe5', b'\xc5\xe5', re.L | re.I)

    assert re.match(b'\xc5', b'\xe5', re.L | re.I) is None

    assert re.match(b'\xe5', b'\xc5', re.L | re.I) is None

    assert re.match(b'(?Li)\xc5\xe5', b'\xc5\xe5')

    assert re.match(b'(?Li)\xc5', b'\xe5') is None

    assert re.match(b'(?Li)\xe5', b'\xc5') is None

def check_interrupt(pattern, string, maxcount):

    class Interrupt(Exception):
        pass
    p = re.compile(pattern)
    for n in range(maxcount):
        try:
            p._fail_after(n, Interrupt)
            p.match(string)
            return n
        except Interrupt:
            pass
        finally:
            p._fail_after(-1, None)
s = '1x\n'
for p in ('[^\\s\\S]', '[^\\d\\D]', '[^\\w\\W]', '[^\\S\\s]'):

    assert re.search(p, s) is None

    assert re.search('(?s:.)' + p, s) is None
print("ReTests::test_character_set_none: ok")
"###);
    assert_output(&out, r###"ReTests::test_character_set_none: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_constants.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_constants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_constants"
# subject = "cpython.test_re.ReTests.test_constants"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_constants
"""Auto-ported test: ReTests::test_constants (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.I == re.IGNORECASE

assert re.L == re.LOCALE

assert re.M == re.MULTILINE

assert re.S == re.DOTALL

assert re.X == re.VERBOSE
print("ReTests::test_constants: ok")
"###);
    assert_output(&out, r###"ReTests::test_constants: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_copying.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_copying() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_copying"
# subject = "cpython.test_re.ReTests.test_copying"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_copying
"""Auto-ported test: ReTests::test_copying (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
import copy
p = re.compile('(?P<int>\\d+)(?:\\.(?P<frac>\\d*))?')

assert copy.copy(p) is p

assert copy.deepcopy(p) is p
m = p.match('12.34')

assert copy.copy(m) is m

assert copy.deepcopy(m) is m
print("ReTests::test_copying: ok")
"###);
    assert_output(&out, r###"ReTests::test_copying: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_expand.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_expand() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_expand"
# subject = "cpython.test_re.ReTests.test_expand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_expand
"""Auto-ported test: ReTests::test_expand (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(?P<first>first) (?P<second>second)', 'first second').expand('\\2 \\1 \\g<second> \\g<first>') == 'second first second first'

assert re.match('(?P<first>first)|(?P<second>second)', 'first').expand('\\2 \\g<second>') == ' '
print("ReTests::test_expand: ok")
"###);
    assert_output(&out, r###"ReTests::test_expand: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_flags.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_flags"
# subject = "cpython.test_re.ReTests.test_flags"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_flags
"""Auto-ported test: ReTests::test_flags (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
for flag in [re.I, re.M, re.X, re.S, re.A, re.U]:

    assert re.compile('^pattern$', flag)
for flag in [re.I, re.M, re.X, re.S, re.A, re.L]:

    assert re.compile(b'^pattern$', flag)
print("ReTests::test_flags: ok")
"###);
    assert_output(&out, r###"ReTests::test_flags: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_fullmatch_possessive_quantifiers.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_fullmatch_possessive_quantifiers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_fullmatch_possessive_quantifiers"
# subject = "cpython.test_re.ReTests.test_fullmatch_possessive_quantifiers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_fullmatch_possessive_quantifiers
"""Auto-ported test: ReTests::test_fullmatch_possessive_quantifiers (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.fullmatch('a++', 'a')

assert re.fullmatch('a*+', 'a')

assert re.fullmatch('a?+', 'a')

assert re.fullmatch('a{1,3}+', 'a')

assert re.fullmatch('a++', 'ab') is None

assert re.fullmatch('a*+', 'ab') is None

assert re.fullmatch('a?+', 'ab') is None

assert re.fullmatch('a{1,3}+', 'ab') is None

assert re.fullmatch('a++b', 'ab')

assert re.fullmatch('a*+b', 'ab')

assert re.fullmatch('a?+b', 'ab')

assert re.fullmatch('a{1,3}+b', 'ab')

assert re.fullmatch('(?:ab)++', 'ab')

assert re.fullmatch('(?:ab)*+', 'ab')

assert re.fullmatch('(?:ab)?+', 'ab')

assert re.fullmatch('(?:ab){1,3}+', 'ab')

assert re.fullmatch('(?:ab)++', 'abc') is None

assert re.fullmatch('(?:ab)*+', 'abc') is None

assert re.fullmatch('(?:ab)?+', 'abc') is None

assert re.fullmatch('(?:ab){1,3}+', 'abc') is None

assert re.fullmatch('(?:ab)++c', 'abc')

assert re.fullmatch('(?:ab)*+c', 'abc')

assert re.fullmatch('(?:ab)?+c', 'abc')

assert re.fullmatch('(?:ab){1,3}+c', 'abc')
print("ReTests::test_fullmatch_possessive_quantifiers: ok")
"###);
    assert_output(&out, r###"ReTests::test_fullmatch_possessive_quantifiers: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_groupdict.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_groupdict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_groupdict"
# subject = "cpython.test_re.ReTests.test_groupdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_groupdict
"""Auto-ported test: ReTests::test_groupdict (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(?P<first>first) (?P<second>second)', 'first second').groupdict() == {'first': 'first', 'second': 'second'}
print("ReTests::test_groupdict: ok")
"###);
    assert_output(&out, r###"ReTests::test_groupdict: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_mark_push_macro_bug.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_mark_push_macro_bug() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_mark_push_macro_bug"
# subject = "cpython.test_re.ReTests.test_MARK_PUSH_macro_bug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_MARK_PUSH_macro_bug
"""Auto-ported test: ReTests::test_MARK_PUSH_macro_bug (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(ab|a)*?b', 'ab').groups() == ('a',)

assert re.match('(ab|a)+?b', 'ab').groups() == ('a',)

assert re.match('(ab|a){0,2}?b', 'ab').groups() == ('a',)

assert re.match('(.b|a)*?b', 'ab').groups() == ('a',)
print("ReTests::test_MARK_PUSH_macro_bug: ok")
"###);
    assert_output(&out, r###"ReTests::test_MARK_PUSH_macro_bug: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_min_until_mark_bug.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_min_until_mark_bug() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_min_until_mark_bug"
# subject = "cpython.test_re.ReTests.test_MIN_UNTIL_mark_bug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_MIN_UNTIL_mark_bug
"""Auto-ported test: ReTests::test_MIN_UNTIL_mark_bug (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
s = 'axxzbcz'
p = '(?:(?:a|bc)*?(xx)??z)*'

assert re.match(p, s).groups() == ('xx',)
s = 'xtcxyzxc'
p = '((x|yz)+?(t)??c)*'
m = re.match(p, s)

assert m.span() == (0, 8)

assert m.span(2) == (6, 7)

assert m.groups() == ('xyzxc', 'x', 't')
print("ReTests::test_MIN_UNTIL_mark_bug: ok")
"###);
    assert_output(&out, r###"ReTests::test_MIN_UNTIL_mark_bug: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_not_literal.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_not_literal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_not_literal"
# subject = "cpython.test_re.ReTests.test_not_literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_not_literal
"""Auto-ported test: ReTests::test_not_literal (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.search('\\s([^a])', ' b').group(1) == 'b'

assert re.search('\\s([^a]*)', ' bb').group(1) == 'bb'
print("ReTests::test_not_literal: ok")
"###);
    assert_output(&out, r###"ReTests::test_not_literal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_search_coverage.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_search_coverage() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_search_coverage"
# subject = "cpython.test_re.ReTests.test_search_coverage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_search_coverage
"""Auto-ported test: ReTests::test_search_coverage (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.search('\\s(b)', ' b').group(1) == 'b'

assert re.search('a\\s', 'a ').group(0) == 'a '
print("ReTests::test_search_coverage: ok")
"###);
    assert_output(&out, r###"ReTests::test_search_coverage: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_search_dot_unicode.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_search_dot_unicode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_search_dot_unicode"
# subject = "cpython.test_re.ReTests.test_search_dot_unicode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_search_dot_unicode
"""Auto-ported test: ReTests::test_search_dot_unicode (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.search('123.*-', '123abc-')

assert re.search('123.*-', '123é-')

assert re.search('123.*-', '123€-')

assert re.search('123.*-', '123\U0010ffff-')

assert re.search('123.*-', '123é€\U0010ffff-')
print("ReTests::test_search_dot_unicode: ok")
"###);
    assert_output(&out, r###"ReTests::test_search_dot_unicode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_search_star_plus.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_search_star_plus() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_search_star_plus"
# subject = "cpython.test_re.ReTests.test_search_star_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_search_star_plus
"""Auto-ported test: ReTests::test_search_star_plus (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.search('x*', 'axx').span(0) == (0, 0)

assert re.search('x*', 'axx').span() == (0, 0)

assert re.search('x+', 'axx').span(0) == (1, 3)

assert re.search('x+', 'axx').span() == (1, 3)

assert re.search('x', 'aaa') is None

assert re.match('a*', 'xxx').span(0) == (0, 0)

assert re.match('a*', 'xxx').span() == (0, 0)

assert re.match('x*', 'xxxa').span(0) == (0, 3)

assert re.match('x*', 'xxxa').span() == (0, 3)

assert re.match('a+', 'xxx') is None
print("ReTests::test_search_star_plus: ok")
"###);
    assert_output(&out, r###"ReTests::test_search_star_plus: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_stack_overflow.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_stack_overflow() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_stack_overflow"
# subject = "cpython.test_re.ReTests.test_stack_overflow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_stack_overflow
"""Auto-ported test: ReTests::test_stack_overflow (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(x)*', 50000 * 'x').group(1) == 'x'

assert re.match('(x)*y', 50000 * 'x' + 'y').group(1) == 'x'

assert re.match('(x)*?y', 50000 * 'x' + 'y').group(1) == 'x'
print("ReTests::test_stack_overflow: ok")
"###);
    assert_output(&out, r###"ReTests::test_stack_overflow: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_unlimited_zero_width_repeat.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_unlimited_zero_width_repeat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_unlimited_zero_width_repeat"
# subject = "cpython.test_re.ReTests.test_unlimited_zero_width_repeat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_unlimited_zero_width_repeat
"""Auto-ported test: ReTests::test_unlimited_zero_width_repeat (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(?:a?)*y', 'z') is None

assert re.match('(?:a?)+y', 'z') is None

assert re.match('(?:a?){2,}y', 'z') is None

assert re.match('(?:a?)*?y', 'z') is None

assert re.match('(?:a?)+?y', 'z') is None

assert re.match('(?:a?){2,}?y', 'z') is None
print("ReTests::test_unlimited_zero_width_repeat: ok")
"###);
    assert_output(&out, r###"ReTests::test_unlimited_zero_width_repeat: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/re_tests__test_weakref.py`.
#[test]
fn test_gen_behavior_std_libs_re_re_tests__test_weakref() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_weakref"
# subject = "cpython.test_re.ReTests.test_weakref"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_weakref
"""Auto-ported test: ReTests::test_weakref (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'
s = 'QabbbcR'
x = re.compile('ab+c')
y = proxy(x)

assert x.findall('QabbbcR') == y.findall('QabbbcR')
print("ReTests::test_weakref: ok")
"###);
    assert_output(&out, r###"ReTests::test_weakref: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/scanner_tokenizes.py`.
#[test]
fn test_gen_behavior_std_libs_re_scanner_tokenizes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "scanner_tokenizes"
# subject = "re.Scanner"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Scanner: re.Scanner walks the input calling per-pattern callbacks (a None callback consumes silently); scan() returns (tokens, remainder)"""
import re

def t_ident(scanner, token):
    return ("ID", token)


def t_int(scanner, token):
    return ("INT", int(token))


def t_op(scanner, token):
    return ("OP", token)


scanner = re.Scanner([
    (r"[a-zA-Z_]\w*", t_ident),
    (r"\d+", t_int),
    (r"[=+\-*/]", t_op),
    (r"\s+", None),            # whitespace: skipped, emits no token
])

tokens, remainder = scanner.scan("sum = 3 * foo + 42")
assert tokens == [
    ("ID", "sum"), ("OP", "="), ("INT", 3), ("OP", "*"),
    ("ID", "foo"), ("OP", "+"), ("INT", 42),
], f"tokens = {tokens!r}"
assert remainder == "", f"remainder = {remainder!r}"

# Unrecognized leading input is left in the remainder.
tokens2, remainder2 = scanner.scan("# ab")
assert tokens2 == [], f"tokens2 = {tokens2!r}"
assert remainder2 == "# ab", f"remainder2 = {remainder2!r}"

print("scanner_tokenizes OK")
"###);
    assert_output(&out, r###"scanner_tokenizes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/search_finds_anywhere.py`.
#[test]
fn test_gen_behavior_std_libs_re_search_finds_anywhere() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "search_finds_anywhere"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.search: re.search finds the first hit anywhere: r'\\d+' on 'abc123def' yields group '123'"""
import re

m = re.search(r"\d+", "abc123def")
assert m is not None, "search found"
assert m.group() == "123", f"group = {m.group()!r}"

print("search_finds_anywhere OK")
"###);
    assert_output(&out, r###"search_finds_anywhere OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/search_reports_span_positions.py`.
#[test]
fn test_gen_behavior_std_libs_re_search_reports_span_positions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "search_reports_span_positions"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.search: the Match object exposes start/end/span: re.search(r'\\d+', 'abc123') gives start 3, end 6, span (3, 6)"""
import re

m = re.search(r"\d+", "abc123")
assert m is not None, "search found"
assert m.start() == 3, f"start = {m.start()!r}"
assert m.end() == 6, f"end = {m.end()!r}"
assert m.span() == (3, 6), f"span = {m.span()!r}"

print("search_reports_span_positions OK")
"###);
    assert_output(&out, r###"search_reports_span_positions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/span_start_end_per_group.py`.
#[test]
fn test_gen_behavior_std_libs_re_span_start_end_per_group() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "span_start_end_per_group"
# subject = "re.Match.span"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.span: span(n)/start(n)/end(n) report per-group offsets: re.search(r'(\\d+)-(\\d+)','id: 12-345') has span()=(4,10), span(1)=(4,6), span(2)=(7,10)"""
import re

m = re.search(r"(\d+)-(\d+)", "id: 12-345")
assert m.span() == (4, 10), f"span() = {m.span()!r}"
assert m.span(1) == (4, 6), f"span(1) = {m.span(1)!r}"
assert m.span(2) == (7, 10), f"span(2) = {m.span(2)!r}"
assert m.start(2) == 7 and m.end(2) == 10, "start/end per group"

print("span_start_end_per_group OK")
"###);
    assert_output(&out, r###"span_start_end_per_group OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/split_captured_separators_kept.py`.
#[test]
fn test_gen_behavior_std_libs_re_split_captured_separators_kept() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "split_captured_separators_kept"
# subject = "re.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.split: a capturing group in the split pattern keeps the separators: re.split(r'(\\d+)','a1b2c3') is ['a','1','b','2','c','3','']"""
import re

assert re.split(r"(\d+)", "a1b2c3") == ["a", "1", "b", "2", "c", "3", ""]

print("split_captured_separators_kept OK")
"###);
    assert_output(&out, r###"split_captured_separators_kept OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/split_maxsplit_limit.py`.
#[test]
fn test_gen_behavior_std_libs_re_split_maxsplit_limit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "split_maxsplit_limit"
# subject = "re.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.split: maxsplit caps the number of splits: re.split(r':','a:b:c:d',maxsplit=2) is ['a','b','c:d']"""
import re

assert re.split(r":", "a:b:c:d") == ["a", "b", "c", "d"], "no limit"
assert re.split(r":", "a:b:c:d", maxsplit=2) == ["a", "b", "c:d"], "maxsplit=2"

print("split_maxsplit_limit OK")
"###);
    assert_output(&out, r###"split_maxsplit_limit OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/split_no_groups.py`.
#[test]
fn test_gen_behavior_std_libs_re_split_no_groups() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "split_no_groups"
# subject = "re.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.split: split breaks on the separator pattern and drops it: re.split(r'\\s+','hello  world  foo') is ['hello','world','foo']"""
import re

assert re.split(r"\s+", "hello  world  foo") == ["hello", "world", "foo"]

print("split_no_groups OK")
"###);
    assert_output(&out, r###"split_no_groups OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/string_start_end_anchors.py`.
#[test]
fn test_gen_behavior_std_libs_re_string_start_end_anchors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "string_start_end_anchors"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.search: \\A anchors at string start and \\Z at string end even under MULTILINE: r'^\\Aabc\\Z$' matches 'abc' but rejects '\\nabc\\n'"""
import re

assert re.search(r"^\Aabc\Z$", "abc", re.M).group(0) == "abc", "\\A..\\Z single line"
assert re.search(r"^\Aabc\Z$", "\nabc\n", re.M) is None, "\\A..\\Z reject newlines under MULTILINE"

print("string_start_end_anchors OK")
"###);
    assert_output(&out, r###"string_start_end_anchors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/sub_callable_replacement.py`.
#[test]
fn test_gen_behavior_std_libs_re_sub_callable_replacement() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_callable_replacement"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: a callable replacement is invoked per match with the Match object: re.sub(r'\\d+', lambda m: str(int(m.group())*2), 'a1b2c3') is 'a2b4c6'"""
import re

doubled = re.sub(r"\d+", lambda m: str(int(m.group()) * 2), "a1b2c3")
assert doubled == "a2b4c6", f"sub callable = {doubled!r}"

print("sub_callable_replacement OK")
"###);
    assert_output(&out, r###"sub_callable_replacement OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/sub_count_limits_replacements.py`.
#[test]
fn test_gen_behavior_std_libs_re_sub_count_limits_replacements() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_count_limits_replacements"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: the count keyword caps replacements: re.sub(r'a','b','aaaaa',count=1) is 'baaaa' while count=0 means replace all"""
import re

assert re.sub(r"a", "b", "aaaaa") == "bbbbb", "count default -> all"
assert re.sub(r"a", "b", "aaaaa", count=0) == "bbbbb", "count=0 -> all"
assert re.sub(r"a", "b", "aaaaa", count=1) == "baaaa", "count=1 -> one"

print("sub_count_limits_replacements OK")
"###);
    assert_output(&out, r###"sub_count_limits_replacements OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/sub_numeric_backreference.py`.
#[test]
fn test_gen_behavior_std_libs_re_sub_numeric_backreference() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_numeric_backreference"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: a numeric backreference in the replacement re-inserts a captured group: re.sub(r'(\\w+)', r'[\\1]', 'hello world') is '[hello] [world]'"""
import re

assert re.sub(r"(\w+)", r"[\1]", "hello world") == "[hello] [world]"

print("sub_numeric_backreference OK")
"###);
    assert_output(&out, r###"sub_numeric_backreference OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/sub_replaces_all_matches.py`.
#[test]
fn test_gen_behavior_std_libs_re_sub_replaces_all_matches() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_replaces_all_matches"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.sub: sub replaces every non-overlapping match by default: r'\\d+' -> 'NUM' on 'abc123def456' is 'abcNUMdefNUM'"""
import re

assert re.sub(r"\d+", "NUM", "abc123def456") == "abcNUMdefNUM"

print("sub_replaces_all_matches OK")
"###);
    assert_output(&out, r###"sub_replaces_all_matches OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/sub_symbolic_group_reference.py`.
#[test]
fn test_gen_behavior_std_libs_re_sub_symbolic_group_reference() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_symbolic_group_reference"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: \\g<name> and \\g<N> symbolic references reinsert groups; an unmatched optional group expands to the empty string"""
import re

# \g<name> reinserts a named group.
assert re.sub(r"(?P<a>x)", r"[\g<a>]", "xx") == "[x][x]", "named \\g ref"
# \g<N> reinserts a numbered group (group swap).
assert re.sub(r"(\w)(\w)", r"\g<2>\g<1>", "ab") == "ba", "numbered \\g ref"
# An unmatched optional group expands to the empty string.
assert re.sub(r"(?P<a>x)|(?P<b>y)", r"\g<b>", "xx") == "", "unmatched group -> empty"

print("sub_symbolic_group_reference OK")
"###);
    assert_output(&out, r###"sub_symbolic_group_reference OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/subn_empty_match_repetition.py`.
#[test]
fn test_gen_behavior_std_libs_re_subn_empty_match_repetition() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "subn_empty_match_repetition"
# subject = "re.subn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.subn: an empty-matching pattern inserts between every character: re.subn(r'b*','x','xyz') is ('xxxyxzx', 4)"""
import re

assert re.subn(r"b*", "x", "xyz") == ("xxxyxzx", 4), "empty matches between chars"
assert re.subn(r"b*", "x", "xyz", count=2) == ("xxxyz", 2), "count limit on empty matches"

print("subn_empty_match_repetition OK")
"###);
    assert_output(&out, r###"subn_empty_match_repetition OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/subn_returns_string_and_count.py`.
#[test]
fn test_gen_behavior_std_libs_re_subn_returns_string_and_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "subn_returns_string_and_count"
# subject = "re.subn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.subn: subn returns (new_string, num_subs): re.subn(r'\\d+','X','a1 b22 c333') is ('aX bX cX', 3); a no-match leaves count 0"""
import re

assert re.subn(r"\d+", "X", "a1 b22 c333") == ("aX bX cX", 3), "subn count"
assert re.subn(r"b+", "x", "xyz") == ("xyz", 0), "subn no match -> count 0"

print("subn_returns_string_and_count OK")
"###);
    assert_output(&out, r###"subn_returns_string_and_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/verbose_ignores_whitespace_and_comments.py`.
#[test]
fn test_gen_behavior_std_libs_re_verbose_ignores_whitespace_and_comments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "verbose_ignores_whitespace_and_comments"
# subject = "re.VERBOSE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.VERBOSE: re.VERBOSE ignores unescaped whitespace and # comments in the pattern; the compiled pattern still reports the VERBOSE flag in .flags"""
import re

vp = re.compile(
    r"""
    \d{4}   # year
    -
    \d{2}   # month
    """,
    re.VERBOSE,
)
assert vp.fullmatch("2024-03") is not None, "verbose matches compact text"
assert vp.fullmatch("2024 - 03") is None, "verbose ignores pattern spaces only"
assert vp.flags & re.VERBOSE, "compiled flags include VERBOSE"

print("verbose_ignores_whitespace_and_comments OK")
"###);
    assert_output(&out, r###"verbose_ignores_whitespace_and_comments OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/word_boundary_anchor.py`.
#[test]
fn test_gen_behavior_std_libs_re_word_boundary_anchor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "word_boundary_anchor"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.search: \\b is a zero-width word boundary: r'\\b\\w+\\b' on '   hello   ' captures 'hello'"""
import re

m = re.search(r"\b\w+\b", "   hello   ")
assert m is not None and m.group() == "hello", f"word boundary group = {m.group() if m else None!r}"

print("word_boundary_anchor OK")
"###);
    assert_output(&out, r###"word_boundary_anchor OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/re/word_boundary_zero_width_count.py`.
#[test]
fn test_gen_behavior_std_libs_re_word_boundary_zero_width_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "word_boundary_zero_width_count"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.findall: \\b is zero-width with one boundary on each side of a word: findall(r'\\b','a') is two hits, findall(r'\\B','a') is zero, and search(r'\\b','') is None"""
import re

assert len(re.findall(r"\b", "a")) == 2, "two boundaries around a single word"
assert len(re.findall(r"\B", "a")) == 0, "no non-boundary inside single char"
assert re.search(r"\b", "") is None, "no boundary in empty string"

print("word_boundary_zero_width_count OK")
"###);
    assert_output(&out, r###"word_boundary_zero_width_count OK
"###);
}
