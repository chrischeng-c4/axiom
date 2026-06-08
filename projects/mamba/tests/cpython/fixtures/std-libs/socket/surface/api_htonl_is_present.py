# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_htonl_is_present"
# subject = "socket.htonl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.htonl: api_htonl_is_present (surface)."""
import socket

assert hasattr(socket, "htonl")
print("api_htonl_is_present OK")
