# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_getservbyname_is_present"
# subject = "socket.getservbyname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.getservbyname: api_getservbyname_is_present (surface)."""
import socket

assert hasattr(socket, "getservbyname")
print("api_getservbyname_is_present OK")
