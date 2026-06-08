# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "urlopen_is_callable"
# subject = "urllib.request.urlopen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.urlopen: urlopen_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.urlopen)
print("urlopen_is_callable OK")
