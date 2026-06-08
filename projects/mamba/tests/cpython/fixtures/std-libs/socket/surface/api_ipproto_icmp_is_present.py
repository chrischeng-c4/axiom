# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipproto_icmp_is_present"
# subject = "socket.IPPROTO_ICMP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPPROTO_ICMP: api_ipproto_icmp_is_present (surface)."""
import socket

assert hasattr(socket, "IPPROTO_ICMP")
print("api_ipproto_icmp_is_present OK")
