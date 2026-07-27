use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/shlex/join_quotes_each_element.py`.
#[test]
fn test_gen_behavior_std_libs_shlex_join_quotes_each_element() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "join_quotes_each_element"
# subject = "shlex.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.join: join applies quote() to every element: safe words pass through, a word with a space is single-quoted, and an empty list joins to the empty string"""
import shlex

assert shlex.join(["a", "b", "c"]) == "a b c", "safe words join with single spaces"
assert shlex.join(["a", "b c", "d"]) == "a 'b c' d", "the spaced element is single-quoted"
assert shlex.join([]) == "", "the empty list joins to the empty string"
print("join_quotes_each_element OK")
"###);
    assert_output(&out, r###"join_quotes_each_element OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shlex/quote_embedded_single_quote.py`.
#[test]
fn test_gen_behavior_std_libs_shlex_quote_embedded_single_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_embedded_single_quote"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: an embedded single quote is escaped with the classic '"'"' splice, e.g. quote("it's") == '\\'it\\'"\\'"\\'s\\''"""
import shlex

# The whole word is single-quoted; the inner ' is broken out as '"'"' so the
# shell rejoins it as a literal single quote.
assert shlex.quote("it's") == "'it'\"'\"'s'", "embedded single quote uses the '\"'\"' splice"
print("quote_embedded_single_quote OK")
"###);
    assert_output(&out, r###"quote_embedded_single_quote OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shlex/quote_empty_string.py`.
#[test]
fn test_gen_behavior_std_libs_shlex_quote_empty_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_empty_string"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: the empty string quotes to the two-character literal '' so it survives a shell word boundary"""
import shlex

assert shlex.quote("") == "''", "empty string quotes to ''"
print("quote_empty_string OK")
"###);
    assert_output(&out, r###"quote_empty_string OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shlex/quote_safe_word_unchanged.py`.
#[test]
fn test_gen_behavior_std_libs_shlex_quote_safe_word_unchanged() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_safe_word_unchanged"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: a word built only from the shell-safe set (letters/digits/@%_-+=:,./) is returned unchanged, no quoting added"""
import shlex

assert shlex.quote("hello_world") == "hello_world", "underscored word is safe"
assert shlex.quote("a-b.c/d:e@f%g=h+i,j") == "a-b.c/d:e@f%g=h+i,j", "every safe punct char passes through"
print("quote_safe_word_unchanged OK")
"###);
    assert_output(&out, r###"quote_safe_word_unchanged OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shlex/quote_wraps_whitespace.py`.
#[test]
fn test_gen_behavior_std_libs_shlex_quote_wraps_whitespace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_wraps_whitespace"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: a value containing whitespace is wrapped in single quotes, e.g. quote('hello world') == "'hello world'" """
import shlex

assert shlex.quote("hello world") == "'hello world'", "space forces single-quoting"
assert shlex.quote("test file name") == "'test file name'", "multiple spaces still one quoted token"
print("quote_wraps_whitespace OK")
"###);
    assert_output(&out, r###"quote_wraps_whitespace OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shlex/split_quoted_segment_is_one_token.py`.
#[test]
fn test_gen_behavior_std_libs_shlex_split_quoted_segment_is_one_token() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "split_quoted_segment_is_one_token"
# subject = "shlex.split"
# kind = "semantic"
# xfail = "mamba shlex.split does not process quotes (returns raw whitespace split); repo-memory project_mamba_stdlib_stub_audit_2026_05_26"
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: a double-quoted segment is collapsed into a single token with the quotes stripped, e.g. split('"hello world" foo') == ['hello world', 'foo']"""
import shlex

assert shlex.split('"hello world" foo') == ["hello world", "foo"], 'double-quoted segment is one token'
assert shlex.split("'a b' c") == ["a b", "c"], "single-quoted segment is one token"
print("split_quoted_segment_is_one_token OK")
"###);
    assert_output(&out, r###"split_quoted_segment_is_one_token OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/shlex/split_whitespace_tokens.py`.
#[test]
fn test_gen_behavior_std_libs_shlex_split_whitespace_tokens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "split_whitespace_tokens"
# subject = "shlex.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: whitespace-separated words split into a token list; the empty string yields an empty list"""
import shlex

assert shlex.split("hello world") == ["hello", "world"], 'split("hello world")'
assert shlex.split("foo   bar    bla") == ["foo", "bar", "bla"], "runs of whitespace collapse"
assert shlex.split("") == [], 'split("")'
print("split_whitespace_tokens OK")
"###);
    assert_output(&out, r###"split_whitespace_tokens OK
"###);
}
