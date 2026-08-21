# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "url2pathname_is_callable"
# subject = "urllib.request.url2pathname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.url2pathname: url2pathname_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.url2pathname)
print("url2pathname_is_callable OK")
