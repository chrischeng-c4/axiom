# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "api_simple_cookie_is_present"
# subject = "http.cookies.SimpleCookie"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.cookies.SimpleCookie: api_simple_cookie_is_present (surface)."""
import http.cookies

assert hasattr(http.cookies, "SimpleCookie")
print("api_simple_cookie_is_present OK")
