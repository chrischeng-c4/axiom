# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "cookie_is_callable"
# subject = "http.cookiejar.Cookie"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.Cookie: cookie_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.Cookie)
print("cookie_is_callable OK")
