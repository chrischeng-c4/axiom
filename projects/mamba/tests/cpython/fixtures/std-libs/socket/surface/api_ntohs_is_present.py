# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ntohs_is_present"
# subject = "socket.ntohs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.ntohs: api_ntohs_is_present (surface)."""
import socket

assert hasattr(socket, "ntohs")
print("api_ntohs_is_present OK")
