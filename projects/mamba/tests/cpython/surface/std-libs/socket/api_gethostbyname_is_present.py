# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_gethostbyname_is_present"
# subject = "socket.gethostbyname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.gethostbyname: api_gethostbyname_is_present (surface)."""
import socket

assert hasattr(socket, "gethostbyname")
print("api_gethostbyname_is_present OK")
