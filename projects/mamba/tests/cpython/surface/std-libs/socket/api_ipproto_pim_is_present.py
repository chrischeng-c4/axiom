# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ipproto_pim_is_present"
# subject = "socket.IPPROTO_PIM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IPPROTO_PIM: api_ipproto_pim_is_present (surface)."""
import socket

assert hasattr(socket, "IPPROTO_PIM")
print("api_ipproto_pim_is_present OK")
