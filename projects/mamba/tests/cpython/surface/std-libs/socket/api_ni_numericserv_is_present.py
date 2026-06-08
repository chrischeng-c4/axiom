# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ni_numericserv_is_present"
# subject = "socket.NI_NUMERICSERV"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.NI_NUMERICSERV: api_ni_numericserv_is_present (surface)."""
import socket

assert hasattr(socket, "NI_NUMERICSERV")
print("api_ni_numericserv_is_present OK")
