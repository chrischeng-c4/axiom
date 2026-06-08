# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ip_pktinfo_is_present"
# subject = "socket.IP_PKTINFO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IP_PKTINFO: api_ip_pktinfo_is_present (surface)."""
import socket

assert hasattr(socket, "IP_PKTINFO")
print("api_ip_pktinfo_is_present OK")
