# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_netmask_spellings"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: a netmask given as prefix int, mask string, dotted netmask, or dotted hostmask all collapse to the prefix form"""
import ipaddress

assert str(ipaddress.IPv4Network(("192.0.2.0", 24))) == "192.0.2.0/24", "net tuple int"
assert str(ipaddress.IPv4Network(("192.0.2.0", "255.255.255.0"))) == "192.0.2.0/24", "net tuple mask"
assert str(ipaddress.IPv4Network("192.0.2.0/255.255.255.0")) == "192.0.2.0/24", "net slash mask"
assert str(ipaddress.IPv4Network("0.0.0.0/0.255.255.255")) == "0.0.0.0/8", "net hostmask"
print("network_netmask_spellings OK")
