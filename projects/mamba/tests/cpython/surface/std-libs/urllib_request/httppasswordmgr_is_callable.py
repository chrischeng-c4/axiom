# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "httppasswordmgr_is_callable"
# subject = "urllib.request.HTTPPasswordMgr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.HTTPPasswordMgr: httppasswordmgr_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.HTTPPasswordMgr)
print("httppasswordmgr_is_callable OK")
