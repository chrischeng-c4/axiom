# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ip_options_is_present"
# subject = "socket.IP_OPTIONS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IP_OPTIONS: api_ip_options_is_present (surface)."""
import socket

assert hasattr(socket, "IP_OPTIONS")
print("api_ip_options_is_present OK")
