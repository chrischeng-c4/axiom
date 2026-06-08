# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_shut_wr_is_present"
# subject = "socket.SHUT_WR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SHUT_WR: api_shut_wr_is_present (surface)."""
import socket

assert hasattr(socket, "SHUT_WR")
print("api_shut_wr_is_present OK")
