# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipproto_ipip_is_present"
# subject = "socket.IPPROTO_IPIP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPPROTO_IPIP: api_ipproto_ipip_is_present (surface)."""
import socket

assert hasattr(socket, "IPPROTO_IPIP")
print("api_ipproto_ipip_is_present OK")
