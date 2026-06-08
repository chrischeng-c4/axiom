# /// script
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
