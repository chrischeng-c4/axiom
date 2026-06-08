# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_sol_ip_is_present"
# subject = "socket.SOL_IP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SOL_IP: api_sol_ip_is_present (surface)."""
import socket

assert hasattr(socket, "SOL_IP")
print("api_sol_ip_is_present OK")
