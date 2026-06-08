# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_response_not_ready_is_present"
# subject = "http.client.ResponseNotReady"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.ResponseNotReady: api_response_not_ready_is_present (surface)."""
import http.client

assert hasattr(http.client, "ResponseNotReady")
print("api_response_not_ready_is_present OK")
