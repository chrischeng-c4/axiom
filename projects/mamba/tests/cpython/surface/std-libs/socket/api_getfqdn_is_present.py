# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_getfqdn_is_present"
# subject = "socket.getfqdn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.getfqdn: api_getfqdn_is_present (surface)."""
import socket

assert hasattr(socket, "getfqdn")
print("api_getfqdn_is_present OK")
