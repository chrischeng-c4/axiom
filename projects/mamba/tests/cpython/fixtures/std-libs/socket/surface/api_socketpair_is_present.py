# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_socketpair_is_present"
# subject = "socket.socketpair"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.socketpair: api_socketpair_is_present (surface)."""
import socket

assert hasattr(socket, "socketpair")
print("api_socketpair_is_present OK")
