# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "basecookie_is_callable"
# subject = "cookies.BaseCookie"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.BaseCookie: basecookie_is_callable (surface)."""
from http import cookies

assert callable(cookies.BaseCookie)
print("basecookie_is_callable OK")
