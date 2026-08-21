# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "ipv4length_attr"
# subject = "ipaddress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress: ipv4length_attr (surface)."""
import ipaddress

assert hasattr(ipaddress, "IPV4LENGTH")
print("ipv4length_attr OK")
