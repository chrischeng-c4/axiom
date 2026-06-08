# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "request_is_callable"
# subject = "urllib.request.Request"
# kind = "mechanical"
# xfail = "urllib.request unimplemented on mamba: urllib.request.Request resolves to None/stub, callable() is False (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.Request: request_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.Request)
print("request_is_callable OK")
