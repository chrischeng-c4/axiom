# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_sock_stream_is_present"
# subject = "socket.SOCK_STREAM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SOCK_STREAM: api_sock_stream_is_present (surface)."""
import socket

assert hasattr(socket, "SOCK_STREAM")
print("api_sock_stream_is_present OK")
