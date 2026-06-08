# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_ip_network_is_present"
# subject = "ipaddress.ip_network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.ip_network: api_ip_network_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "ip_network")
print("api_ip_network_is_present OK")
