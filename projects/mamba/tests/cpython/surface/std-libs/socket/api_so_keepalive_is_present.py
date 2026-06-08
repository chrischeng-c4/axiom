# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_so_keepalive_is_present"
# subject = "socket.SO_KEEPALIVE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SO_KEEPALIVE: api_so_keepalive_is_present (surface)."""
import socket

assert hasattr(socket, "SO_KEEPALIVE")
print("api_so_keepalive_is_present OK")
