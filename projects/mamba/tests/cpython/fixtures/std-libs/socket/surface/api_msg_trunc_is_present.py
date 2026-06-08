# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_msg_trunc_is_present"
# subject = "socket.MSG_TRUNC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.MSG_TRUNC: api_msg_trunc_is_present (surface)."""
import socket

assert hasattr(socket, "MSG_TRUNC")
print("api_msg_trunc_is_present OK")
