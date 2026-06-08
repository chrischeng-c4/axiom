# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_ipv6_length_is_present"
# subject = "ipaddress.IPV6LENGTH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.IPV6LENGTH: api_ipv6_length_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "IPV6LENGTH")
print("api_ipv6_length_is_present OK")
