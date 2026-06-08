# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ip_retopts_is_present"
# subject = "socket.IP_RETOPTS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IP_RETOPTS: api_ip_retopts_is_present (surface)."""
import socket

assert hasattr(socket, "IP_RETOPTS")
print("api_ip_retopts_is_present OK")
