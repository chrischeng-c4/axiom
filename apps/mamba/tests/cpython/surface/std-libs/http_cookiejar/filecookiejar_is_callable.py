# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "filecookiejar_is_callable"
# subject = "http.cookiejar.FileCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.FileCookieJar: filecookiejar_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.FileCookieJar)
print("filecookiejar_is_callable OK")
