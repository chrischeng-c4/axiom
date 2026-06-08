# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipv6_rthdr_type_0_is_present"
# subject = "socket.IPV6_RTHDR_TYPE_0"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPV6_RTHDR_TYPE_0: api_ipv6_rthdr_type_0_is_present (surface)."""
import socket

assert hasattr(socket, "IPV6_RTHDR_TYPE_0")
print("api_ipv6_rthdr_type_0_is_present OK")
