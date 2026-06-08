# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ethertype_arp_is_present"
# subject = "socket.ETHERTYPE_ARP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.ETHERTYPE_ARP: api_ethertype_arp_is_present (surface)."""
import socket

assert hasattr(socket, "ETHERTYPE_ARP")
print("api_ethertype_arp_is_present OK")
