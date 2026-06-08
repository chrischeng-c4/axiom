# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "api_cookie_error_is_present"
# subject = "http.cookies.CookieError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.cookies.CookieError: api_cookie_error_is_present (surface)."""
import http.cookies

assert hasattr(http.cookies, "CookieError")
print("api_cookie_error_is_present OK")
