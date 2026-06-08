# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_herror_is_present"
# subject = "socket.herror"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.herror: api_herror_is_present (surface)."""
import socket

assert hasattr(socket, "herror")
print("api_herror_is_present OK")
