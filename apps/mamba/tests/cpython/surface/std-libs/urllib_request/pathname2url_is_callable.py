# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "pathname2url_is_callable"
# subject = "urllib.request.pathname2url"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.pathname2url: pathname2url_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.pathname2url)
print("pathname2url_is_callable OK")
