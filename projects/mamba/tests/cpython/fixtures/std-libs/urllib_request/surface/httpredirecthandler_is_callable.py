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
# xfail = "urllib.request unimplemented on mamba: urllib.request.HTTPRedirectHandler resolves to None/stub (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.HTTPRedirectHandler: httpredirecthandler_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.HTTPRedirectHandler)
print("httpredirecthandler_is_callable OK")
