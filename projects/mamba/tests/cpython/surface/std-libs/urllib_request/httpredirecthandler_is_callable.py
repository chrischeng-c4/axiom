# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "httpredirecthandler_is_callable"
# subject = "urllib.request.HTTPRedirectHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.HTTPRedirectHandler: httpredirecthandler_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.HTTPRedirectHandler)
print("httpredirecthandler_is_callable OK")
