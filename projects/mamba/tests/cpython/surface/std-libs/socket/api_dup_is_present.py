# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_dup_is_present"
# subject = "socket.dup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.dup: api_dup_is_present (surface)."""
import socket

assert hasattr(socket, "dup")
print("api_dup_is_present OK")
