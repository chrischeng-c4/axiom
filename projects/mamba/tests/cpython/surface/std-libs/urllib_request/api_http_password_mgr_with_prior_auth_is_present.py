# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_http_password_mgr_with_prior_auth_is_present"
# subject = "urllib.request.HTTPPasswordMgrWithPriorAuth"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.HTTPPasswordMgrWithPriorAuth: api_http_password_mgr_with_prior_auth_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "HTTPPasswordMgrWithPriorAuth")
print("api_http_password_mgr_with_prior_auth_is_present OK")
