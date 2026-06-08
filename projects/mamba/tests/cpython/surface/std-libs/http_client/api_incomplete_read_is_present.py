# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_incomplete_read_is_present"
# subject = "http.client.IncompleteRead"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.IncompleteRead: api_incomplete_read_is_present (surface)."""
import http.client

assert hasattr(http.client, "IncompleteRead")
print("api_incomplete_read_is_present OK")
