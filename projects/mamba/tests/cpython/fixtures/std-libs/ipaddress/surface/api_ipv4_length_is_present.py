# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_ipv4_length_is_present"
# subject = "ipaddress.IPV4LENGTH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.IPV4LENGTH: api_ipv4_length_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "IPV4LENGTH")
print("api_ipv4_length_is_present OK")
