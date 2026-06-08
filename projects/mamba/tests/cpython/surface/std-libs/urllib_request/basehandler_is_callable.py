# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "basehandler_is_callable"
# subject = "urllib.request.BaseHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.BaseHandler: basehandler_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.BaseHandler)
print("basehandler_is_callable OK")
