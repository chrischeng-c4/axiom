# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ni_numerichost_is_present"
# subject = "socket.NI_NUMERICHOST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.NI_NUMERICHOST: api_ni_numerichost_is_present (surface)."""
import socket

assert hasattr(socket, "NI_NUMERICHOST")
print("api_ni_numerichost_is_present OK")
