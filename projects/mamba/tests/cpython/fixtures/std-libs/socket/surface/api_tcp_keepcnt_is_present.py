# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_tcp_keepcnt_is_present"
# subject = "socket.TCP_KEEPCNT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.TCP_KEEPCNT: api_tcp_keepcnt_is_present (surface)."""
import socket

assert hasattr(socket, "TCP_KEEPCNT")
print("api_tcp_keepcnt_is_present OK")
