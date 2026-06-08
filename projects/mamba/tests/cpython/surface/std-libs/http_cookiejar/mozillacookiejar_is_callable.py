# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "mozillacookiejar_is_callable"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: mozillacookiejar_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.MozillaCookieJar)
print("mozillacookiejar_is_callable OK")
