# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "httpbasicauthhandler_is_callable"
# subject = "urllib.request.HTTPBasicAuthHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.HTTPBasicAuthHandler: httpbasicauthhandler_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.HTTPBasicAuthHandler)
print("httpbasicauthhandler_is_callable OK")
