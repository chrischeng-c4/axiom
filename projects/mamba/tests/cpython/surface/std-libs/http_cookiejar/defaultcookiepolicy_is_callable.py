# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "defaultcookiepolicy_is_callable"
# subject = "http.cookiejar.DefaultCookiePolicy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.DefaultCookiePolicy: defaultcookiepolicy_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.DefaultCookiePolicy)
print("defaultcookiepolicy_is_callable OK")
