# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_getaddrinfo_is_present"
# subject = "socket.getaddrinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.getaddrinfo: api_getaddrinfo_is_present (surface)."""
import socket

assert hasattr(socket, "getaddrinfo")
print("api_getaddrinfo_is_present OK")
