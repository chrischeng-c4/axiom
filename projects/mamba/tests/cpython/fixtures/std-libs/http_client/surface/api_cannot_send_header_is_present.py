# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_cannot_send_header_is_present"
# subject = "http.client.CannotSendHeader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.CannotSendHeader: api_cannot_send_header_is_present (surface)."""
import http.client

assert hasattr(http.client, "CannotSendHeader")
print("api_cannot_send_header_is_present OK")
