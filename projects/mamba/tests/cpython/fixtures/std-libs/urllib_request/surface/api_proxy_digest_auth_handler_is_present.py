# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_proxy_digest_auth_handler_is_present"
# subject = "urllib.request.ProxyDigestAuthHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.ProxyDigestAuthHandler: api_proxy_digest_auth_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "ProxyDigestAuthHandler")
print("api_proxy_digest_auth_handler_is_present OK")
