# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_gethostbyaddr_is_present"
# subject = "socket.gethostbyaddr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.gethostbyaddr: api_gethostbyaddr_is_present (surface)."""
import socket

assert hasattr(socket, "gethostbyaddr")
print("api_gethostbyaddr_is_present OK")
