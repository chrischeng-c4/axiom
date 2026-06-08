# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_socket_kind_is_present"
# subject = "socket.SocketKind"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SocketKind: api_socket_kind_is_present (surface)."""
import socket

assert hasattr(socket, "SocketKind")
print("api_socket_kind_is_present OK")
