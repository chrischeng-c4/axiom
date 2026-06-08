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
# xfail = "urllib.request unimplemented on mamba: urllib.request.HTTPCookieProcessor resolves to None/stub (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.HTTPCookieProcessor: httpcookieprocessor_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.HTTPCookieProcessor)
print("httpcookieprocessor_is_callable OK")
