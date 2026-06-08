# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_has_dualstack_ipv6_is_present"
# subject = "socket.has_dualstack_ipv6"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.has_dualstack_ipv6: api_has_dualstack_ipv6_is_present (surface)."""
import socket

assert hasattr(socket, "has_dualstack_ipv6")
print("api_has_dualstack_ipv6_is_present OK")
