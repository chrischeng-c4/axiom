# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ip_block_source_is_present"
# subject = "socket.IP_BLOCK_SOURCE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IP_BLOCK_SOURCE: api_ip_block_source_is_present (surface)."""
import socket

assert hasattr(socket, "IP_BLOCK_SOURCE")
print("api_ip_block_source_is_present OK")
