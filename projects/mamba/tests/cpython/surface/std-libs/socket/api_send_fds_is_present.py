# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_send_fds_is_present"
# subject = "socket.send_fds"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.send_fds: api_send_fds_is_present (surface)."""
import socket

assert hasattr(socket, "send_fds")
print("api_send_fds_is_present OK")
