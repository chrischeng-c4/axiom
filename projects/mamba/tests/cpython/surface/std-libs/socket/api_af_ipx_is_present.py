# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_af_ipx_is_present"
# subject = "socket.AF_IPX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AF_IPX: api_af_ipx_is_present (surface)."""
import socket

assert hasattr(socket, "AF_IPX")
print("api_af_ipx_is_present OK")
