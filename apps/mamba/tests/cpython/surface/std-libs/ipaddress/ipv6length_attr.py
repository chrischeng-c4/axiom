# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv6length_attr"
# subject = "ipaddress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress: ipv6length_attr (surface)."""
import ipaddress

assert hasattr(ipaddress, "IPV6LENGTH")
print("ipv6length_attr OK")
