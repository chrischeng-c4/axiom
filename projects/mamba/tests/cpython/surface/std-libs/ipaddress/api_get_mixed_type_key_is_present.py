# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_get_mixed_type_key_is_present"
# subject = "ipaddress.get_mixed_type_key"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.get_mixed_type_key: api_get_mixed_type_key_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "get_mixed_type_key")
print("api_get_mixed_type_key_is_present OK")
