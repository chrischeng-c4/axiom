# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_tcp_fastopen_is_present"
# subject = "socket.TCP_FASTOPEN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.TCP_FASTOPEN: api_tcp_fastopen_is_present (surface)."""
import socket

assert hasattr(socket, "TCP_FASTOPEN")
print("api_tcp_fastopen_is_present OK")
