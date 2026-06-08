# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_capi_is_present"
# subject = "socket.CAPI"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.CAPI: api_capi_is_present (surface)."""
import socket

assert hasattr(socket, "CAPI")
print("api_capi_is_present OK")
