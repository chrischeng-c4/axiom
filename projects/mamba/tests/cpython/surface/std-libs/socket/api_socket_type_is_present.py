# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_socket_type_is_present"
# subject = "socket.SocketType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SocketType: api_socket_type_is_present (surface)."""
import socket

assert hasattr(socket, "SocketType")
print("api_socket_type_is_present OK")
