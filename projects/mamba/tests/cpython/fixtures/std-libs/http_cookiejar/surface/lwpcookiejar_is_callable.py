# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "lwpcookiejar_is_callable"
# subject = "http.cookiejar.LWPCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.LWPCookieJar: lwpcookiejar_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.LWPCookieJar)
print("lwpcookiejar_is_callable OK")
