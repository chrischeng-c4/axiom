# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_so_useloopback_is_present"
# subject = "socket.SO_USELOOPBACK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SO_USELOOPBACK: api_so_useloopback_is_present (surface)."""
import socket

assert hasattr(socket, "SO_USELOOPBACK")
print("api_so_useloopback_is_present OK")
