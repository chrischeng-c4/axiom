# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_fromfd_is_present"
# subject = "socket.fromfd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.fromfd: api_fromfd_is_present (surface)."""
import socket

assert hasattr(socket, "fromfd")
print("api_fromfd_is_present OK")
