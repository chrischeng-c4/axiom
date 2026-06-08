# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_somaxconn_is_present"
# subject = "socket.SOMAXCONN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SOMAXCONN: api_somaxconn_is_present (surface)."""
import socket

assert hasattr(socket, "SOMAXCONN")
print("api_somaxconn_is_present OK")
