# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_af_de_cnet_is_present"
# subject = "socket.AF_DECnet"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AF_DECnet: api_af_de_cnet_is_present (surface)."""
import socket

assert hasattr(socket, "AF_DECnet")
print("api_af_de_cnet_is_present OK")
