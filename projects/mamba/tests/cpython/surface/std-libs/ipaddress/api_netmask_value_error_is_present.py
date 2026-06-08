# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_netmask_value_error_is_present"
# subject = "ipaddress.NetmaskValueError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.NetmaskValueError: api_netmask_value_error_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "NetmaskValueError")
print("api_netmask_value_error_is_present OK")
