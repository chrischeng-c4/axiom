# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_create_server_is_present"
# subject = "socket.create_server"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.create_server: api_create_server_is_present (surface)."""
import socket

assert hasattr(socket, "create_server")
print("api_create_server_is_present OK")
