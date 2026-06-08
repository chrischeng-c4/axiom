# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_invalid_url_is_present"
# subject = "http.client.InvalidURL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.InvalidURL: api_invalid_url_is_present (surface)."""
import http.client

assert hasattr(http.client, "InvalidURL")
print("api_invalid_url_is_present OK")
