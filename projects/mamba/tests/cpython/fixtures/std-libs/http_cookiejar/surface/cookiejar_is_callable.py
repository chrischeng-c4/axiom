# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "cookiejar_is_callable"
# subject = "http.cookiejar.CookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.CookieJar: cookiejar_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.CookieJar)
print("cookiejar_is_callable OK")
