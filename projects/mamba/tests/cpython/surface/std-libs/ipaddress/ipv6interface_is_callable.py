# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv6interface_is_callable"
# subject = "ipaddress.IPv6Interface"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv6Interface: ipv6interface_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.IPv6Interface)
print("ipv6interface_is_callable OK")
