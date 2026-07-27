use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/http_cookies/load_bare_comma_raises.py`.
#[test]
fn test_gen_errors_std_libs_http_cookies_load_bare_comma_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "errors"
# case = "load_bare_comma_raises"
# subject = "cookies.SimpleCookie"
# kind = "mechanical"
# xfail = "mamba SimpleCookie shell has no bound load(); does not parse or raise CookieError (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: load_bare_comma_raises (errors)."""
from http import cookies

_raised = False
try:
    cookies.SimpleCookie().load('a=b; c,d=e')
except cookies.CookieError:
    _raised = True
assert _raised, "load_bare_comma_raises: expected cookies.CookieError"
print("load_bare_comma_raises OK")
"###);
    assert_output(&out, r###"load_bare_comma_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookies/missing_key_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_http_cookies_missing_key_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "errors"
# case = "missing_key_raises_keyerror"
# subject = "cookies.SimpleCookie"
# kind = "mechanical"
# xfail = "mamba SimpleCookie shell is not a real dict; missing-key lookup returns None instead of raising KeyError (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: missing_key_raises_keyerror (errors)."""
from http import cookies

_raised = False
try:
    cookies.SimpleCookie()['never_set']
except KeyError:
    _raised = True
assert _raised, "missing_key_raises_keyerror: expected KeyError"
print("missing_key_raises_keyerror OK")
"###);
    assert_output(&out, r###"missing_key_raises_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookies/morsel_set_illegal_chars_raises.py`.
#[test]
fn test_gen_errors_std_libs_http_cookies_morsel_set_illegal_chars_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "errors"
# case = "morsel_set_illegal_chars_raises"
# subject = "cookies.Morsel"
# kind = "mechanical"
# xfail = "mamba Morsel shell has no bound set(); illegal-char guard is absent (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: morsel_set_illegal_chars_raises (errors)."""
from http import cookies

_raised = False
try:
    cookies.Morsel().set('foo bar', 'v', 'v')
except cookies.CookieError:
    _raised = True
assert _raised, "morsel_set_illegal_chars_raises: expected cookies.CookieError"
print("morsel_set_illegal_chars_raises OK")
"###);
    assert_output(&out, r###"morsel_set_illegal_chars_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookies/morsel_set_reserved_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_http_cookies_morsel_set_reserved_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "errors"
# case = "morsel_set_reserved_name_raises"
# subject = "cookies.Morsel"
# kind = "mechanical"
# xfail = "mamba Morsel shell has no bound set(); reserved-name guard is absent (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: morsel_set_reserved_name_raises (errors)."""
from http import cookies

_raised = False
try:
    cookies.Morsel().set('expires', 'v', 'v')
except cookies.CookieError:
    _raised = True
assert _raised, "morsel_set_reserved_name_raises: expected cookies.CookieError"
print("morsel_set_reserved_name_raises OK")
"###);
    assert_output(&out, r###"morsel_set_reserved_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookies/name_with_control_char_raises.py`.
#[test]
fn test_gen_errors_std_libs_http_cookies_name_with_control_char_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "errors"
# case = "name_with_control_char_raises"
# subject = "cookies.SimpleCookie"
# kind = "mechanical"
# xfail = "mamba http.cookies SimpleCookie is a passive shell; __setitem__ does not validate names (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: name_with_control_char_raises (errors)."""
from http import cookies

_raised = False
try:
    cookies.SimpleCookie().__setitem__('bad\x01name', 'value')
except cookies.CookieError:
    _raised = True
assert _raised, "name_with_control_char_raises: expected cookies.CookieError"
print("name_with_control_char_raises OK")
"###);
    assert_output(&out, r###"name_with_control_char_raises OK
"###);
}
