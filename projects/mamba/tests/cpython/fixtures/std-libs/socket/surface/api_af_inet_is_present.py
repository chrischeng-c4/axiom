# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_af_inet_is_present"
# subject = "socket.AF_INET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AF_INET: api_af_inet_is_present (surface)."""
import socket

assert hasattr(socket, "AF_INET")
print("api_af_inet_is_present OK")
