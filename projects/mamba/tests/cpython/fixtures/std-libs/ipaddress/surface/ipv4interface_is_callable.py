# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv4interface_is_callable"
# subject = "ipaddress.IPv4Interface"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Interface: ipv4interface_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.IPv4Interface)
print("ipv4interface_is_callable OK")
