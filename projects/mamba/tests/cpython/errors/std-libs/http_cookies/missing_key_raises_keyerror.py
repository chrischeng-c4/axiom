# /// script
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
