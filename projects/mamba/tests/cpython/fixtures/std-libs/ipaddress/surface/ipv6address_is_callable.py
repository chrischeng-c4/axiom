# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv6address_is_callable"
# subject = "ipaddress.IPv6Address"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv6Address: ipv6address_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.IPv6Address)
print("ipv6address_is_callable OK")
