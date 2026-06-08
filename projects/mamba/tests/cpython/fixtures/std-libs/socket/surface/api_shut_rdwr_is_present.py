# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_shut_rdwr_is_present"
# subject = "socket.SHUT_RDWR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SHUT_RDWR: api_shut_rdwr_is_present (surface)."""
import socket

assert hasattr(socket, "SHUT_RDWR")
print("api_shut_rdwr_is_present OK")
