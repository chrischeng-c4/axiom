# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipv6_use_min_mtu_is_present"
# subject = "socket.IPV6_USE_MIN_MTU"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPV6_USE_MIN_MTU: api_ipv6_use_min_mtu_is_present (surface)."""
import socket

assert hasattr(socket, "IPV6_USE_MIN_MTU")
print("api_ipv6_use_min_mtu_is_present OK")
