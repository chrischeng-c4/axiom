# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_sock_raw_is_present"
# subject = "socket.SOCK_RAW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SOCK_RAW: api_sock_raw_is_present (surface)."""
import socket

assert hasattr(socket, "SOCK_RAW")
print("api_sock_raw_is_present OK")
