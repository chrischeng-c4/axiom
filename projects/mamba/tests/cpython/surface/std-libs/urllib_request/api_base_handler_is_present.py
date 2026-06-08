# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_base_handler_is_present"
# subject = "urllib.request.BaseHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.BaseHandler: api_base_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "BaseHandler")
print("api_base_handler_is_present OK")
