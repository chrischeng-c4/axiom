# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_interface_netmask_spellings"
# subject = "ipaddress.IPv4Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Interface: IPv4Interface netmask spellings (tuple int/str, dotted mask, slash mask) all collapse to the prefix form"""
import ipaddress

assert str(ipaddress.IPv4Interface(("192.0.2.0", 24))) == "192.0.2.0/24", "tuple int"
assert str(ipaddress.IPv4Interface(("192.0.2.0", "24"))) == "192.0.2.0/24", "tuple str"
assert str(ipaddress.IPv4Interface(("192.0.2.0", "255.255.255.0"))) == "192.0.2.0/24", "tuple mask"
assert str(ipaddress.IPv4Interface("192.0.2.0/255.255.255.0")) == "192.0.2.0/24", "slash mask"
print("ipv4_interface_netmask_spellings OK")
