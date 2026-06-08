# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_msg_nosignal_is_present"
# subject = "socket.MSG_NOSIGNAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.MSG_NOSIGNAL: api_msg_nosignal_is_present (surface)."""
import socket

assert hasattr(socket, "MSG_NOSIGNAL")
print("api_msg_nosignal_is_present OK")
