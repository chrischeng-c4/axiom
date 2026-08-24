use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/cookie_tests__test_secure_httponly_false_if_not_present.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_cookie_tests__test_secure_httponly_false_if_not_present() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "cookie_tests__test_secure_httponly_false_if_not_present"
# subject = "cpython.test_http_cookies.CookieTests.test_secure_httponly_false_if_not_present"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_http_cookies.py::CookieTests::test_secure_httponly_false_if_not_present
"""Auto-ported test: CookieTests::test_secure_httponly_false_if_not_present (CPython 3.12 oracle)."""


import copy
import unittest
import doctest
from http import cookies
import pickle
from test import support


def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(cookies))
    return tests


# --- test body ---
C = cookies.SimpleCookie()
C.load('eggs=scrambled; Path=/bacon')

assert not C['eggs']['httponly']

assert not C['eggs']['secure']
print("CookieTests::test_secure_httponly_false_if_not_present: ok")
"###);
    assert_output(&out, r###"CookieTests::test_secure_httponly_false_if_not_present: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/load_collapses_surrounding_whitespace.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_load_collapses_surrounding_whitespace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_collapses_surrounding_whitespace"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: load() collapses surrounding whitespace around keys/values/attrs; output() is the canonical compact form"""
from http import cookies

c = cookies.SimpleCookie()
c.load("eggs  =  scrambled  ;  secure  ;  path  =  bar   ; foo=foo   ")
assert c.output() == "Set-Cookie: eggs=scrambled; Path=bar; Secure\r\nSet-Cookie: foo=foo", \
    f"extra-space output = {c.output()!r}"
print("load_collapses_surrounding_whitespace OK")
"###);
    assert_output(&out, r###"load_collapses_surrounding_whitespace OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/load_full_metadata_and_js_output.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_load_full_metadata_and_js_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_full_metadata_and_js_output"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output()/js_output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: load() captures Version/Path metadata, output([attr]) filters, and js_output() emits the fixed <script> document.cookie wrapper"""
from http import cookies

c = cookies.SimpleCookie()
c.load('Customer="WILE_E_COYOTE"; Version=1; Path=/acme')
assert c["Customer"]["version"] == "1", f"version = {c['Customer']['version']!r}"
assert c.output(["path"]) == 'Set-Cookie: Customer="WILE_E_COYOTE"; Path=/acme', \
    f"filtered output = {c.output(['path'])!r}"
expected_js = (
    '\n        <script type="text/javascript">'
    '\n        <!-- begin hiding'
    '\n        document.cookie = "Customer=\\"WILE_E_COYOTE\\"; Path=/acme; Version=1";'
    '\n        // end hiding -->'
    '\n        </script>\n        '
)
assert c.js_output() == expected_js, f"js_output = {c.js_output()!r}"
print("load_full_metadata_and_js_output OK")
"###);
    assert_output(&out, r###"load_full_metadata_and_js_output OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/load_non_cookie_string_parses_empty.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_load_non_cookie_string_parses_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_non_cookie_string_parses_empty"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: a non-cookie string (Set-Cookie:, version-only, leading bare flag) parses to an empty cookie with empty output()"""
from http import cookies

c = cookies.SimpleCookie()
for bad in ("Set-Cookie: foo=bar", "foo=bar; baz", "secure;foo=bar", "Version=1;foo=bar"):
    c.load(bad)
    assert dict(c) == {}, f"non-cookie {bad!r} parsed empty"
    assert c.output() == "", f"non-cookie {bad!r} empty output"
print("load_non_cookie_string_parses_empty OK")
"###);
    assert_output(&out, r###"load_non_cookie_string_parses_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/load_parses_cookie_header.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_load_parses_cookie_header() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_parses_cookie_header"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load(); no parsing (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: load() parses a 'k1=v1; k2=v2; k3=v3' Cookie header into one Morsel per key"""
from http import cookies

c = cookies.SimpleCookie()
c.load("name=John; age=30; city=NYC")
assert c["name"].value == "John", f"name = {c['name'].value!r}"
assert c["age"].value == "30", f"age = {c['age'].value!r}"
assert c["city"].value == "NYC", f"city = {c['city'].value!r}"
print("load_parses_cookie_header OK")
"###);
    assert_output(&out, r###"load_parses_cookie_header OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/load_secure_httponly_truthiness.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_load_secure_httponly_truthiness() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_secure_httponly_truthiness"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load(); attributes are not populated (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: secure/httponly are falsy when absent from a loaded cookie and keep their explicit string value when present"""
from http import cookies

c = cookies.SimpleCookie()
c.load("eggs=scrambled; Path=/bacon")
assert not c["eggs"]["httponly"], "httponly absent is falsy"
assert not c["eggs"]["secure"], "secure absent is falsy"
c2 = cookies.SimpleCookie()
c2.load("eggs=scrambled; httponly=foo; secure=bar; Path=/bacon")
assert c2["eggs"]["httponly"] == "foo", f"httponly value = {c2['eggs']['httponly']!r}"
assert c2["eggs"]["secure"] == "bar", f"secure value = {c2['eggs']['secure']!r}"
print("load_secure_httponly_truthiness OK")
"###);
    assert_output(&out, r###"load_secure_httponly_truthiness OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/morsel_copy_is_equal_but_distinct.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_morsel_copy_is_equal_but_distinct() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_copy_is_equal_but_distinct"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell has no bound copy() and no value equality (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.Morsel: Morsel.copy() and copy.copy() return an equal-but-distinct Morsel"""
from http import cookies

import copy

m = cookies.Morsel()
m.set("foo", "bar", "baz")
m.update({"version": 2, "comment": "foo"})
for dup in (m.copy(), copy.copy(m)):
    assert isinstance(dup, cookies.Morsel), "copy is a Morsel"
    assert dup is not m and dup == m, "copy is distinct but equal"
print("morsel_copy_is_equal_but_distinct OK")
"###);
    assert_output(&out, r###"morsel_copy_is_equal_but_distinct OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/morsel_js_output_assignment.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_morsel_js_output_assignment() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_js_output_assignment"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell has no bound js_output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: Morsel.js_output() emits a JavaScript document.cookie assignment carrying the key and value"""
from http import cookies

c = cookies.SimpleCookie()
c["js_key"] = "js_val"
js = c["js_key"].js_output()
assert isinstance(js, str), f"js_output type = {type(js)!r}"
assert "js_key" in js, f"key in js_output: {js!r}"
assert "js_val" in js, f"value in js_output: {js!r}"
print("morsel_js_output_assignment OK")
"###);
    assert_output(&out, r###"morsel_js_output_assignment OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/morsel_outputstring_has_no_header.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_morsel_outputstring_has_no_header() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_outputstring_has_no_header"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell has no bound OutputString() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: Morsel.OutputString() returns 'key=value' without the leading 'Set-Cookie:' header"""
from http import cookies

c = cookies.SimpleCookie()
c["item"] = "value"
morsel = c["item"]
os = morsel.OutputString()
assert isinstance(os, str), f"OutputString type = {type(os)!r}"
assert "item=value" in os, f"OutputString has item=value: {os!r}"
assert "Set-Cookie:" not in os, f"OutputString lacks Set-Cookie: {os!r}"
print("morsel_outputstring_has_no_header OK")
"###);
    assert_output(&out, r###"morsel_outputstring_has_no_header OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/output_emits_set_cookie_line.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_output_emits_set_cookie_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_emits_set_cookie_line"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: output() serializes a cookie to a 'Set-Cookie: key=value' header line"""
from http import cookies

c = cookies.SimpleCookie()
c["session"] = "s123"
out = c.output()
assert out.startswith("Set-Cookie:"), f"output starts with Set-Cookie: {out!r}"
assert "session=s123" in out, f"session=s123 in output: {out!r}"
print("output_emits_set_cookie_line OK")
"###);
    assert_output(&out, r###"output_emits_set_cookie_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/output_filtered_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_output_filtered_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_filtered_attrs"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: output(attrs) restricts the rendered Morsel attributes to the named subset"""
from http import cookies

c = cookies.SimpleCookie()
c.load('Customer="WILE_E_COYOTE"; Version=1; Path=/acme')
assert c.output(["path"]) == 'Set-Cookie: Customer="WILE_E_COYOTE"; Path=/acme', \
    f"filtered output = {c.output(['path'])!r}"
print("output_filtered_attrs OK")
"###);
    assert_output(&out, r###"output_filtered_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/output_flags_alphabetical_order.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_output_flags_alphabetical_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_flags_alphabetical_order"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render ordered flag tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: setting secure+httponly emits valueless flags in alphabetical order: '; HttpOnly; Secure'"""
from http import cookies

c = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
c["Customer"]["secure"] = True
c["Customer"]["httponly"] = True
assert c.output() == 'Set-Cookie: Customer="WILE_E_COYOTE"; HttpOnly; Secure', \
    f"flag output = {c.output()!r}"
print("output_flags_alphabetical_order OK")
"###);
    assert_output(&out, r###"output_flags_alphabetical_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/output_includes_path_and_domain.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_output_includes_path_and_domain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_includes_path_and_domain"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render attribute tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: Path and Domain Morsel attributes render as 'Path=...' / 'Domain=...' tokens in output()"""
from http import cookies

c = cookies.SimpleCookie()
c["auth"] = "token123"
c["auth"]["path"] = "/api"
c["auth"]["domain"] = ".example.com"
out = c.output()
assert "Path=/api" in out, f"Path in output: {out!r}"
assert "Domain=.example.com" in out, f"Domain in output: {out!r}"
print("output_includes_path_and_domain OK")
"###);
    assert_output(&out, r###"output_includes_path_and_domain OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/output_includes_secure_and_httponly_flags.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_output_includes_secure_and_httponly_flags() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_includes_secure_and_httponly_flags"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render flag tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: secure and httponly Morsel flags render as valueless 'Secure' / 'HttpOnly' tokens in output()"""
from http import cookies

c = cookies.SimpleCookie()
c["secure_cookie"] = "val"
c["secure_cookie"]["secure"] = True
c["secure_cookie"]["httponly"] = True
out = c.output()
assert "Secure" in out, f"Secure in output: {out!r}"
assert "HttpOnly" in out, f"HttpOnly in output: {out!r}"
print("output_includes_secure_and_httponly_flags OK")
"###);
    assert_output(&out, r###"output_includes_secure_and_httponly_flags OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/output_max_age_and_expires.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_output_max_age_and_expires() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_max_age_and_expires"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render Max-Age/Expires tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: max-age renders as 'Max-Age=N' and expires=0 renders an absolute GMT date in output()"""
from http import cookies

c = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
c["Customer"]["max-age"] = 10
assert c.output() == 'Set-Cookie: Customer="WILE_E_COYOTE"; Max-Age=10', \
    f"max-age output = {c.output()!r}"
c2 = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
c2["Customer"]["expires"] = 0
assert c2.output().endswith("GMT"), f"expires output = {c2.output()!r}"
print("output_max_age_and_expires OK")
"###);
    assert_output(&out, r###"output_max_age_and_expires OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/output_sep_joins_lines.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_output_sep_joins_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_sep_joins_lines"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: output(sep=...) joins multiple Set-Cookie lines with the chosen separator"""
from http import cookies

c = cookies.SimpleCookie()
c.load("chips=ahoy; vienna=finger")
assert c.output(sep="\n") == "Set-Cookie: chips=ahoy\nSet-Cookie: vienna=finger", \
    f"sep output = {c.output(sep=chr(10))!r}"
print("output_sep_joins_lines OK")
"###);
    assert_output(&out, r###"output_sep_joins_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/repr_shows_quoted_values.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_repr_shows_quoted_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "repr_shows_quoted_values"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load() and a generic repr (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: repr(SimpleCookie) lists each key with its single-quoted value: <SimpleCookie: chips='ahoy' vienna='finger'>"""
from http import cookies

c = cookies.SimpleCookie()
c.load("chips=ahoy; vienna=finger")
assert repr(c) == "<SimpleCookie: chips='ahoy' vienna='finger'>", f"repr = {c!r}"
print("repr_shows_quoted_values OK")
"###);
    assert_output(&out, r###"repr_shows_quoted_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/simplecookie_stores_multiple_values.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_simplecookie_stores_multiple_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "simplecookie_stores_multiple_values"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell drops __setitem__ values and len(); no real dict storage (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: SimpleCookie holds multiple key=value cookies and reports len() and each Morsel's .value"""
from http import cookies

c = cookies.SimpleCookie()
c["user"] = "alice"
c["token"] = "xyz789"
assert len(c) == 2, f"cookie count = {len(c)!r}"
assert c["user"].value == "alice", f"user = {c['user'].value!r}"
assert c["token"].value == "xyz789", f"token = {c['token'].value!r}"
print("simplecookie_stores_multiple_values OK")
"###);
    assert_output(&out, r###"simplecookie_stores_multiple_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/value_comma_semicolon_octal_escaped.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_value_comma_semicolon_octal_escaped() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "value_comma_semicolon_octal_escaped"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not octal-escape delimiters in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: comma and semicolon inside a value become \\054 and \\073 inside the quoted output()"""
from http import cookies

c = cookies.SimpleCookie()
c["val"] = "some,funky;stuff"
assert c.output(["val"]) == 'Set-Cookie: val="some\\054funky\\073stuff"', \
    f"extended = {c.output(['val'])!r}"
print("value_comma_semicolon_octal_escaped OK")
"###);
    assert_output(&out, r###"value_comma_semicolon_octal_escaped OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/value_newline_octal_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_value_newline_octal_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "value_newline_octal_roundtrip"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output() octal round-trip (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: a literal newline in a loaded value round-trips through \\012 octal encoding between load() and output()"""
from http import cookies

c = cookies.SimpleCookie()
c.load('keebler="E=mc2; L=\\"Loves\\"; fudge=\\012;"')
assert c["keebler"].value == 'E=mc2; L="Loves"; fudge=\n;', \
    f"keebler value = {c['keebler'].value!r}"
assert c.output() == 'Set-Cookie: keebler="E=mc2; L=\\"Loves\\"; fudge=\\012;"', \
    f"keebler output = {c.output()!r}"
print("value_newline_octal_roundtrip OK")
"###);
    assert_output(&out, r###"value_newline_octal_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/http_cookies/value_non_ascii_octal_escaped.py`.
#[test]
fn test_gen_behavior_std_libs_http_cookies_value_non_ascii_octal_escaped() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "value_non_ascii_octal_escaped"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not octal-escape values in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: a non-ASCII value (U+00A9) is backslash-octal escaped (\\251) and double-quoted in output()"""
from http import cookies

c = cookies.SimpleCookie()
c["foo"] = "\u00a9"
assert str(c["foo"]) == 'Set-Cookie: foo="\\251"', f"non-ascii = {str(c['foo'])!r}"
c["foo"]["comment"] = "comment \u00a9"
assert str(c["foo"]) == 'Set-Cookie: foo="\\251"; Comment="comment \\251"', \
    f"comment = {str(c['foo'])!r}"
print("value_non_ascii_octal_escaped OK")
"###);
    assert_output(&out, r###"value_non_ascii_octal_escaped OK
"###);
}
