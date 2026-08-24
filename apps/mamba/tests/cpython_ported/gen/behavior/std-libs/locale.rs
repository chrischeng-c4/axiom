use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/locale/error_is_exception.py`.
#[test]
fn test_gen_behavior_std_libs_locale_error_is_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "error_is_exception"
# subject = "locale.Error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.Error: locale.Error is a subclass of Exception"""
import locale

assert issubclass(locale.Error, Exception), "locale.Error is an Exception subclass"

print("error_is_exception OK")
"###);
    assert_output(&out, r###"error_is_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/format_string_mapping_passthrough.py`.
#[test]
fn test_gen_behavior_std_libs_locale_format_string_mapping_passthrough() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_mapping_passthrough"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string with a %(name)s mapping passes straight through to plain %-formatting"""
import locale

# A %(name)s mapping passes straight through to %-formatting. Without
# grouping the output matches plain %-formatting, keeping these
# assertions locale-independent.
assert (
    locale.format_string("%(foo)s bing.", {"foo": "bar"})
    == "%(foo)s bing." % {"foo": "bar"}
), "format_string mapping with trailing text"
assert (
    locale.format_string("%(foo)s", {"foo": "bar"})
    == "%(foo)s" % {"foo": "bar"}
), "format_string bare mapping"

print("format_string_mapping_passthrough OK")
"###);
    assert_output(&out, r###"format_string_mapping_passthrough OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/format_string_mapping_with_escape.py`.
#[test]
fn test_gen_behavior_std_libs_locale_format_string_mapping_with_escape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_mapping_with_escape"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string with a mapping and an escaped %% literal matches plain %-formatting"""
import locale

# Mapping with an escaped %% literal.
assert (
    locale.format_string("%(foo)s %%d", {"foo": "bar"})
    == "%(foo)s %%d" % {"foo": "bar"}
), "format_string mapping with percent escape"

print("format_string_mapping_with_escape OK")
"###);
    assert_output(&out, r###"format_string_mapping_with_escape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/format_string_percent_escape.py`.
#[test]
fn test_gen_behavior_std_libs_locale_format_string_percent_escape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_percent_escape"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string preserves a literal %% escape and formats a single float arg like %-formatting"""
import locale

# Literal %% survives, and a single float arg formats like %-formatting.
assert locale.format_string("%f%%", 1.0) == "%f%%" % 1.0, "percent escape, single arg"
assert locale.format_string("%f%%", 1.0) == "1.000000%", "percent escape value"

print("format_string_percent_escape OK")
"###);
    assert_output(&out, r###"format_string_percent_escape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/format_string_tuple_args.py`.
#[test]
fn test_gen_behavior_std_libs_locale_format_string_tuple_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_tuple_args"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string with positional tuple args plus an escaped %% mid-string matches plain %-formatting"""
import locale

# Positional tuple args plus an escaped %% mid-string.
assert (
    locale.format_string("%d %f%%d", (1, 1.0))
    == "%d %f%%d" % (1, 1.0)
), "format_string tuple args"

print("format_string_tuple_args OK")
"###);
    assert_output(&out, r###"format_string_tuple_args OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/getencoding_resolves_to_codec.py`.
#[test]
fn test_gen_behavior_std_libs_locale_getencoding_resolves_to_codec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "getencoding_resolves_to_codec"
# subject = "locale.getencoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.getencoding: getencoding returns a non-empty codec-resolvable encoding name"""
import codecs
import locale

enc = locale.getencoding()
assert isinstance(enc, str), "getencoding -> str"
assert enc != "", "getencoding non-empty"
codecs.lookup(enc)  # raises LookupError if not a real codec

print("getencoding_resolves_to_codec OK")
"###);
    assert_output(&out, r###"getencoding_resolves_to_codec OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/getlocale_returns_two_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_locale_getlocale_returns_two_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "getlocale_returns_two_tuple"
# subject = "locale.getlocale"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.getlocale: getlocale() returns a (language, encoding) 2-tuple"""
import locale

loc = locale.getlocale()
assert isinstance(loc, tuple), "getlocale -> tuple"
assert len(loc) == 2, "getlocale -> 2-tuple"

print("getlocale_returns_two_tuple OK")
"###);
    assert_output(&out, r###"getlocale_returns_two_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/getpreferredencoding_resolves_to_codec.py`.
#[test]
fn test_gen_behavior_std_libs_locale_getpreferredencoding_resolves_to_codec() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "getpreferredencoding_resolves_to_codec"
# subject = "locale.getpreferredencoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.getpreferredencoding: getpreferredencoding returns a str; if non-empty it resolves via codecs.lookup"""
import codecs
import locale

pref = locale.getpreferredencoding()
assert isinstance(pref, str), "getpreferredencoding -> str"
if pref:
    codecs.lookup(pref)

print("getpreferredencoding_resolves_to_codec OK")
"###);
    assert_output(&out, r###"getpreferredencoding_resolves_to_codec OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/parse_localename_bare_encoding.py`.
#[test]
fn test_gen_behavior_std_libs_locale_parse_localename_bare_encoding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "parse_localename_bare_encoding"
# subject = "locale._parse_localename"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale._parse_localename: _parse_localename of a bare encoding 'UTF-8' yields (None, 'UTF-8')"""
import locale

# _parse_localename splits "<lang>.<encoding>"; a bare encoding yields
# (None, encoding).
assert locale._parse_localename("UTF-8") == (None, "UTF-8"), "_parse_localename UTF-8"

print("parse_localename_bare_encoding OK")
"###);
    assert_output(&out, r###"parse_localename_bare_encoding OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/setlocale_query_returns_str.py`.
#[test]
fn test_gen_behavior_std_libs_locale_setlocale_query_returns_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "setlocale_query_returns_str"
# subject = "locale.setlocale"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.setlocale: setlocale(category) with no locale arg queries (does not change) and returns a str for every standard LC_ category"""
import locale

# setlocale with only a category queries (does not change) the current
# value, returning a string for every standard LC_ category.
for cat in (
    locale.LC_ALL,
    locale.LC_CTYPE,
    locale.LC_TIME,
    locale.LC_COLLATE,
    locale.LC_MONETARY,
    locale.LC_NUMERIC,
):
    current = locale.setlocale(cat)
    assert isinstance(current, str), "setlocale query -> str"

print("setlocale_query_returns_str OK")
"###);
    assert_output(&out, r###"setlocale_query_returns_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/strcoll_orders_like_c.py`.
#[test]
fn test_gen_behavior_std_libs_locale_strcoll_orders_like_c() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "strcoll_orders_like_c"
# subject = "locale.strcoll"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strcoll: strcoll returns <0, 0, >0 for a<b, a==a, b>a in the C locale (byte ordering)"""
import locale

# In the default 'C' locale strcoll reduces to byte ordering and is
# deterministic, so no locale-gating is needed.
assert locale.strcoll("a", "b") < 0, "strcoll a<b"
assert locale.strcoll("a", "a") == 0, "strcoll a==a"
assert locale.strcoll("b", "a") > 0, "strcoll b>a"

print("strcoll_orders_like_c OK")
"###);
    assert_output(&out, r###"strcoll_orders_like_c OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/locale/strxfrm_preserves_order.py`.
#[test]
fn test_gen_behavior_std_libs_locale_strxfrm_preserves_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "strxfrm_preserves_order"
# subject = "locale.strxfrm"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strxfrm: strxfrm maps a string to a str sort key whose comparison preserves the original order"""
import locale

# strxfrm maps a string to a sort key; transformed keys preserve order.
assert locale.strxfrm("a") < locale.strxfrm("b"), "strxfrm a<b"
assert isinstance(locale.strxfrm("a"), str), "strxfrm -> str"

print("strxfrm_preserves_order OK")
"###);
    assert_output(&out, r###"strxfrm_preserves_order OK
"###);
}
