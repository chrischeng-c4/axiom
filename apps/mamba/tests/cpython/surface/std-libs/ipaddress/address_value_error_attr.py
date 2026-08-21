# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "address_value_error_attr"
# subject = "ipaddress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress: address_value_error_attr (surface)."""
import ipaddress

assert hasattr(ipaddress, "AddressValueError")
print("address_value_error_attr OK")
