# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_collapse_addresses_is_present"
# subject = "ipaddress.collapse_addresses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.collapse_addresses: api_collapse_addresses_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "collapse_addresses")
print("api_collapse_addresses_is_present OK")
