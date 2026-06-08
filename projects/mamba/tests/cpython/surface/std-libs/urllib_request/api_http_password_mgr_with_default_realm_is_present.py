# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_http_password_mgr_with_default_realm_is_present"
# subject = "urllib.request.HTTPPasswordMgrWithDefaultRealm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.HTTPPasswordMgrWithDefaultRealm: api_http_password_mgr_with_default_realm_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "HTTPPasswordMgrWithDefaultRealm")
print("api_http_password_mgr_with_default_realm_is_present OK")
