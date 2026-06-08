# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv4network_is_callable"
# subject = "ipaddress.IPv4Network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: ipv4network_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.IPv4Network)
print("ipv4network_is_callable OK")
