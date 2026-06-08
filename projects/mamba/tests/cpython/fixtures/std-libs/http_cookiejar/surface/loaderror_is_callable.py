# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "loaderror_is_callable"
# subject = "http.cookiejar.LoadError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar.LoadError: loaderror_is_callable (surface)."""
import http.cookiejar

assert callable(http.cookiejar.LoadError)
print("loaderror_is_callable OK")
