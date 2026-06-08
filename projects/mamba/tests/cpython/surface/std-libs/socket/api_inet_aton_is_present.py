# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_inet_aton_is_present"
# subject = "socket.inet_aton"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.inet_aton: api_inet_aton_is_present (surface)."""
import socket

assert hasattr(socket, "inet_aton")
print("api_inet_aton_is_present OK")
