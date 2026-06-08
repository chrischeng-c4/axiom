# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_pf_system_is_present"
# subject = "socket.PF_SYSTEM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.PF_SYSTEM: api_pf_system_is_present (surface)."""
import socket

assert hasattr(socket, "PF_SYSTEM")
print("api_pf_system_is_present OK")
