# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ip_add_source_membership_is_present"
# subject = "socket.IP_ADD_SOURCE_MEMBERSHIP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.IP_ADD_SOURCE_MEMBERSHIP: api_ip_add_source_membership_is_present (surface)."""
import socket

assert hasattr(socket, "IP_ADD_SOURCE_MEMBERSHIP")
print("api_ip_add_source_membership_is_present OK")
