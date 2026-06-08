# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "simplecookie_is_callable"
# subject = "cookies.SimpleCookie"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: simplecookie_is_callable (surface)."""
from http import cookies

assert callable(cookies.SimpleCookie)
print("simplecookie_is_callable OK")
