# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "request_is_callable"
# subject = "urllib.request.Request"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.Request: request_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.Request)
print("request_is_callable OK")
