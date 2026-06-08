# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_improper_connection_state_is_present"
# subject = "http.client.ImproperConnectionState"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.ImproperConnectionState: api_improper_connection_state_is_present (surface)."""
import http.client

assert hasattr(http.client, "ImproperConnectionState")
print("api_improper_connection_state_is_present OK")
