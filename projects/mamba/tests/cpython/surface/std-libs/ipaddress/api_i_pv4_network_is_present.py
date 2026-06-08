# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_i_pv4_network_is_present"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.IPv4Network: api_i_pv4_network_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "IPv4Network")
print("api_i_pv4_network_is_present OK")
