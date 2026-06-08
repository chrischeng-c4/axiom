# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_address_family_is_present"
# subject = "socket.AddressFamily"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AddressFamily: api_address_family_is_present (surface)."""
import socket

assert hasattr(socket, "AddressFamily")
print("api_address_family_is_present OK")
