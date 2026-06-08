# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_ip_interface_is_present"
# subject = "ipaddress.ip_interface"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.ip_interface: api_ip_interface_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "ip_interface")
print("api_ip_interface_is_present OK")
