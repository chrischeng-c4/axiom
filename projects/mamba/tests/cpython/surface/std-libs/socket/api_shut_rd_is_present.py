# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_shut_rd_is_present"
# subject = "socket.SHUT_RD"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SHUT_RD: api_shut_rd_is_present (surface)."""
import socket

assert hasattr(socket, "SHUT_RD")
print("api_shut_rd_is_present OK")
