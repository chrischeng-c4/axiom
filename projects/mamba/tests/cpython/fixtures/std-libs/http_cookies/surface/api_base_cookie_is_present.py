# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "api_base_cookie_is_present"
# subject = "http.cookies.BaseCookie"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.cookies.BaseCookie: api_base_cookie_is_present (surface)."""
import http.cookies

assert hasattr(http.cookies, "BaseCookie")
print("api_base_cookie_is_present OK")
