# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_if_nametoindex_is_present"
# subject = "socket.if_nametoindex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.if_nametoindex: api_if_nametoindex_is_present (surface)."""
import socket

assert hasattr(socket, "if_nametoindex")
print("api_if_nametoindex_is_present OK")
