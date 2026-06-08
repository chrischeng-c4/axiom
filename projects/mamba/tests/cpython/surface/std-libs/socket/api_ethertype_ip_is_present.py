# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ethertype_ip_is_present"
# subject = "socket.ETHERTYPE_IP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.ETHERTYPE_IP: api_ethertype_ip_is_present (surface)."""
import socket

assert hasattr(socket, "ETHERTYPE_IP")
print("api_ethertype_ip_is_present OK")
