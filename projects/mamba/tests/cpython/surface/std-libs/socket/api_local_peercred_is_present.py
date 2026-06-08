# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_local_peercred_is_present"
# subject = "socket.LOCAL_PEERCRED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.LOCAL_PEERCRED: api_local_peercred_is_present (surface)."""
import socket

assert hasattr(socket, "LOCAL_PEERCRED")
print("api_local_peercred_is_present OK")
