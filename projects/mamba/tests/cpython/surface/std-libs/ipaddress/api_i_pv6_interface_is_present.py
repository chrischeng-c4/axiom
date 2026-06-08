# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_i_pv6_interface_is_present"
# subject = "ipaddress.IPv6Interface"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.IPv6Interface: api_i_pv6_interface_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "IPv6Interface")
print("api_i_pv6_interface_is_present OK")
