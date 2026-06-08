# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_v6_int_to_packed_is_present"
# subject = "ipaddress.v6_int_to_packed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.v6_int_to_packed: api_v6_int_to_packed_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "v6_int_to_packed")
print("api_v6_int_to_packed_is_present OK")
