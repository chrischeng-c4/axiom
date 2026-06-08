# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_gaierror_is_present"
# subject = "socket.gaierror"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.gaierror: api_gaierror_is_present (surface)."""
import socket

assert hasattr(socket, "gaierror")
print("api_gaierror_is_present OK")
