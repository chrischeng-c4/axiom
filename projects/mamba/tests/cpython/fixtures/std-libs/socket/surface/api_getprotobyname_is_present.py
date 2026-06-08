# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_getprotobyname_is_present"
# subject = "socket.getprotobyname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.getprotobyname: api_getprotobyname_is_present (surface)."""
import socket

assert hasattr(socket, "getprotobyname")
print("api_getprotobyname_is_present OK")
