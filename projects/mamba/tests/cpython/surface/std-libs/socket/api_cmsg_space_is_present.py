# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_cmsg_space_is_present"
# subject = "socket.CMSG_SPACE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.CMSG_SPACE: api_cmsg_space_is_present (surface)."""
import socket

assert hasattr(socket, "CMSG_SPACE")
print("api_cmsg_space_is_present OK")
