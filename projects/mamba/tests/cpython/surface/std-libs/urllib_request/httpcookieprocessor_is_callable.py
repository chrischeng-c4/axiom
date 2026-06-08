# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "httpcookieprocessor_is_callable"
# subject = "urllib.request.HTTPCookieProcessor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.HTTPCookieProcessor: httpcookieprocessor_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.HTTPCookieProcessor)
print("httpcookieprocessor_is_callable OK")
