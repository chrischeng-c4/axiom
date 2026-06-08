# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "api_mozilla_cookie_jar_is_present"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: api_mozilla_cookie_jar_is_present (surface)."""
import http.cookiejar

assert hasattr(http.cookiejar, "MozillaCookieJar")
print("api_mozilla_cookie_jar_is_present OK")
