# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_tcp_notsent_lowat_is_present"
# subject = "socket.TCP_NOTSENT_LOWAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.TCP_NOTSENT_LOWAT: api_tcp_notsent_lowat_is_present (surface)."""
import socket

assert hasattr(socket, "TCP_NOTSENT_LOWAT")
print("api_tcp_notsent_lowat_is_present OK")
