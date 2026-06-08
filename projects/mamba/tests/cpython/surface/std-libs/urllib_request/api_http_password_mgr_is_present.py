# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_http_password_mgr_is_present"
# subject = "urllib.request.HTTPPasswordMgr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.HTTPPasswordMgr: api_http_password_mgr_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "HTTPPasswordMgr")
print("api_http_password_mgr_is_present OK")
