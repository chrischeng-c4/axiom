# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "collapse_addresses_is_callable"
# subject = "ipaddress.collapse_addresses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.collapse_addresses: collapse_addresses_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.collapse_addresses)
print("collapse_addresses_is_callable OK")
