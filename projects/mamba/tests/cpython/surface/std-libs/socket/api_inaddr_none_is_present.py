# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_inaddr_none_is_present"
# subject = "socket.INADDR_NONE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.INADDR_NONE: api_inaddr_none_is_present (surface)."""
import socket

assert hasattr(socket, "INADDR_NONE")
print("api_inaddr_none_is_present OK")
