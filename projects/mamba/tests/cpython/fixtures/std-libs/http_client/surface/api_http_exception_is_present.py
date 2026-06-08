# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_http_exception_is_present"
# subject = "http.client.HTTPException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.HTTPException: api_http_exception_is_present (surface)."""
import http.client

assert hasattr(http.client, "HTTPException")
print("api_http_exception_is_present OK")
