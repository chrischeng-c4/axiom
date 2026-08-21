# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv4address_is_callable"
# subject = "ipaddress.IPv4Address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Address: ipv4address_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.IPv4Address)
print("ipv4address_is_callable OK")
