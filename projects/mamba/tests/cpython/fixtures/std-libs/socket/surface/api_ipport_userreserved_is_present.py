# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipport_userreserved_is_present"
# subject = "socket.IPPORT_USERRESERVED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPPORT_USERRESERVED: api_ipport_userreserved_is_present (surface)."""
import socket

assert hasattr(socket, "IPPORT_USERRESERVED")
print("api_ipport_userreserved_is_present OK")
