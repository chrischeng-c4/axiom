# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ip_network_is_callable"
# subject = "ipaddress.ip_network"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_network: ip_network_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.ip_network)
print("ip_network_is_callable OK")
