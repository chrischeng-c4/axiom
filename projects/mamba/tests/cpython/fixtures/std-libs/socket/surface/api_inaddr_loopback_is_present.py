# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_inaddr_loopback_is_present"
# subject = "socket.INADDR_LOOPBACK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.INADDR_LOOPBACK: api_inaddr_loopback_is_present (surface)."""
import socket

assert hasattr(socket, "INADDR_LOOPBACK")
print("api_inaddr_loopback_is_present OK")
