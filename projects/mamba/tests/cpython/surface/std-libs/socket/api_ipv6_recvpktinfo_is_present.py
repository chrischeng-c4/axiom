# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipv6_recvpktinfo_is_present"
# subject = "socket.IPV6_RECVPKTINFO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPV6_RECVPKTINFO: api_ipv6_recvpktinfo_is_present (surface)."""
import socket

assert hasattr(socket, "IPV6_RECVPKTINFO")
print("api_ipv6_recvpktinfo_is_present OK")
