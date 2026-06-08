# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_proxy_basic_auth_handler_is_present"
# subject = "urllib.request.ProxyBasicAuthHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.ProxyBasicAuthHandler: api_proxy_basic_auth_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "ProxyBasicAuthHandler")
print("api_proxy_basic_auth_handler_is_present OK")
