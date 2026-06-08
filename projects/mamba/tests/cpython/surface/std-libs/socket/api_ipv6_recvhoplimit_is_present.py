# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipv6_recvhoplimit_is_present"
# subject = "socket.IPV6_RECVHOPLIMIT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPV6_RECVHOPLIMIT: api_ipv6_recvhoplimit_is_present (surface)."""
import socket

assert hasattr(socket, "IPV6_RECVHOPLIMIT")
print("api_ipv6_recvhoplimit_is_present OK")
