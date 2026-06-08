# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "api_cookie_is_present"
# subject = "http.cookiejar.Cookie"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.cookiejar.Cookie: api_cookie_is_present (surface)."""
import http.cookiejar

assert hasattr(http.cookiejar, "Cookie")
print("api_cookie_is_present OK")
