# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipport_reserved_is_present"
# subject = "socket.IPPORT_RESERVED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPPORT_RESERVED: api_ipport_reserved_is_present (surface)."""
import socket

assert hasattr(socket, "IPPORT_RESERVED")
print("api_ipport_reserved_is_present OK")
