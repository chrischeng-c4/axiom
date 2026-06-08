# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_line_too_long_is_present"
# subject = "http.client.LineTooLong"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.LineTooLong: api_line_too_long_is_present (surface)."""
import http.client

assert hasattr(http.client, "LineTooLong")
print("api_line_too_long_is_present OK")
