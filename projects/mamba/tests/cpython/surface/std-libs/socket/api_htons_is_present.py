# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_htons_is_present"
# subject = "socket.htons"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.htons: api_htons_is_present (surface)."""
import socket

assert hasattr(socket, "htons")
print("api_htons_is_present OK")
