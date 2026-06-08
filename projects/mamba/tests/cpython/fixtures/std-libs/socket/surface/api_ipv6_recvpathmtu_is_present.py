# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipv6_recvpathmtu_is_present"
# subject = "socket.IPV6_RECVPATHMTU"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPV6_RECVPATHMTU: api_ipv6_recvpathmtu_is_present (surface)."""
import socket

assert hasattr(socket, "IPV6_RECVPATHMTU")
print("api_ipv6_recvpathmtu_is_present OK")
