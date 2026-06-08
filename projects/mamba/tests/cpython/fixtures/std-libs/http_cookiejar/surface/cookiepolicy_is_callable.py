# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "cookiepolicy_is_callable"
# subject = "http.cookiejar.CookiePolicy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.CookiePolicy: cookiepolicy_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.CookiePolicy)
print("cookiepolicy_is_callable OK")
