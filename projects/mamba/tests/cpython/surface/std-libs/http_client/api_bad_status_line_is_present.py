# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_bad_status_line_is_present"
# subject = "http.client.BadStatusLine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.BadStatusLine: api_bad_status_line_is_present (surface)."""
import http.client

assert hasattr(http.client, "BadStatusLine")
print("api_bad_status_line_is_present OK")
