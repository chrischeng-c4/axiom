# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_if_nameindex_is_present"
# subject = "socket.if_nameindex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.if_nameindex: api_if_nameindex_is_present (surface)."""
import socket

assert hasattr(socket, "if_nameindex")
print("api_if_nameindex_is_present OK")
