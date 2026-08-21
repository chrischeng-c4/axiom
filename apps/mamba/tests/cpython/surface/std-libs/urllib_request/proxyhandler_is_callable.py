# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "proxyhandler_is_callable"
# subject = "urllib.request.ProxyHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.ProxyHandler: proxyhandler_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.ProxyHandler)
print("proxyhandler_is_callable OK")
