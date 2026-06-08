# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_http_response_is_present"
# subject = "http.client.HTTPResponse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.HTTPResponse: api_http_response_is_present (surface)."""
import http.client

assert hasattr(http.client, "HTTPResponse")
print("api_http_response_is_present OK")
