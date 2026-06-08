# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ethertype_vlan_is_present"
# subject = "socket.ETHERTYPE_VLAN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.ETHERTYPE_VLAN: api_ethertype_vlan_is_present (surface)."""
import socket

assert hasattr(socket, "ETHERTYPE_VLAN")
print("api_ethertype_vlan_is_present OK")
