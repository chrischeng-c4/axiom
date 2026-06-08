# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "api_default_cookie_policy_is_present"
# subject = "http.cookiejar.DefaultCookiePolicy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.cookiejar.DefaultCookiePolicy: api_default_cookie_policy_is_present (surface)."""
import http.cookiejar

assert hasattr(http.cookiejar, "DefaultCookiePolicy")
print("api_default_cookie_policy_is_present OK")
