# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_inet_ntop_is_present"
# subject = "socket.inet_ntop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.inet_ntop: api_inet_ntop_is_present (surface)."""
import socket

assert hasattr(socket, "inet_ntop")
print("api_inet_ntop_is_present OK")
