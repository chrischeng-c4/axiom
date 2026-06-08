# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_so_rcvtimeo_is_present"
# subject = "socket.SO_RCVTIMEO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SO_RCVTIMEO: api_so_rcvtimeo_is_present (surface)."""
import socket

assert hasattr(socket, "SO_RCVTIMEO")
print("api_so_rcvtimeo_is_present OK")
