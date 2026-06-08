# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_msg_eof_is_present"
# subject = "socket.MSG_EOF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.MSG_EOF: api_msg_eof_is_present (surface)."""
import socket

assert hasattr(socket, "MSG_EOF")
print("api_msg_eof_is_present OK")
