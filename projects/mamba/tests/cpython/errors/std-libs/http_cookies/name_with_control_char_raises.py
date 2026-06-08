# /// script
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
