# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "api_remote_disconnected_is_present"
# subject = "http.client.RemoteDisconnected"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.client.RemoteDisconnected: api_remote_disconnected_is_present (surface)."""
import http.client

assert hasattr(http.client, "RemoteDisconnected")
print("api_remote_disconnected_is_present OK")
