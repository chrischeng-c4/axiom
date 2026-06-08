# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_https_connection_is_present"
# subject = "http.client.HTTPSConnection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.HTTPSConnection: api_https_connection_is_present (surface)."""
import http.client

assert hasattr(http.client, "HTTPSConnection")
print("api_https_connection_is_present OK")
