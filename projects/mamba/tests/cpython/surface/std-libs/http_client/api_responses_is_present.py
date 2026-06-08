# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_responses_is_present"
# subject = "http.client.responses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.responses: api_responses_is_present (surface)."""
import http.client

assert hasattr(http.client, "responses")
print("api_responses_is_present OK")
