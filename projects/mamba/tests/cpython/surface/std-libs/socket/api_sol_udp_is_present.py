# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_sol_udp_is_present"
# subject = "socket.SOL_UDP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SOL_UDP: api_sol_udp_is_present (surface)."""
import socket

assert hasattr(socket, "SOL_UDP")
print("api_sol_udp_is_present OK")
