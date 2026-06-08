# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_unknown_protocol_is_present"
# subject = "http.client.UnknownProtocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.UnknownProtocol: api_unknown_protocol_is_present (surface)."""
import http.client

assert hasattr(http.client, "UnknownProtocol")
print("api_unknown_protocol_is_present OK")
