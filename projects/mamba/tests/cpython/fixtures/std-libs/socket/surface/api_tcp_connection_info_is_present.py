# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_tcp_connection_info_is_present"
# subject = "socket.TCP_CONNECTION_INFO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.TCP_CONNECTION_INFO: api_tcp_connection_info_is_present (surface)."""
import socket

assert hasattr(socket, "TCP_CONNECTION_INFO")
print("api_tcp_connection_info_is_present OK")
