# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ip_address_is_callable"
# subject = "ipaddress.ip_address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: ip_address_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.ip_address)
print("ip_address_is_callable OK")
