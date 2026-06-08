# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_msg_ctrunc_is_present"
# subject = "socket.MSG_CTRUNC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.MSG_CTRUNC: api_msg_ctrunc_is_present (surface)."""
import socket

assert hasattr(socket, "MSG_CTRUNC")
print("api_msg_ctrunc_is_present OK")
