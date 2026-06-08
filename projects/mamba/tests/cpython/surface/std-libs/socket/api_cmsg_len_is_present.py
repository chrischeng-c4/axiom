# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_cmsg_len_is_present"
# subject = "socket.CMSG_LEN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.CMSG_LEN: api_cmsg_len_is_present (surface)."""
import socket

assert hasattr(socket, "CMSG_LEN")
print("api_cmsg_len_is_present OK")
