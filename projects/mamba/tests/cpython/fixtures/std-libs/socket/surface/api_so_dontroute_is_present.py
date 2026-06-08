# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_so_dontroute_is_present"
# subject = "socket.SO_DONTROUTE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SO_DONTROUTE: api_so_dontroute_is_present (surface)."""
import socket

assert hasattr(socket, "SO_DONTROUTE")
print("api_so_dontroute_is_present OK")
