# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv6network_is_callable"
# subject = "ipaddress.IPv6Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv6Network: ipv6network_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.IPv6Network)
print("ipv6network_is_callable OK")
