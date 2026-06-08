# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "httphandler_is_class"
# subject = "urllib.request.HTTPHandler"
# kind = "mechanical"
# xfail = "urllib.request unimplemented on mamba: urllib.request.HTTPHandler resolves to None/stub, not a class (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.HTTPHandler: httphandler_is_class (surface)."""
import urllib.request

assert type(urllib.request.HTTPHandler).__name__ == "type"
print("httphandler_is_class OK")
