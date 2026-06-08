# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_interface_prefix_netmask_roundtrip"
# subject = "ipaddress.IPv4Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Interface: for every IPv4 prefix length 0..32 the prefix and netmask spellings of an interface round-trip to the same text"""
import ipaddress

for i in range(0, 33):
    base = "0.0.0.0/%d" % i
    iface = ipaddress.IPv4Interface(base)
    assert str(iface) == base, ("prefix", i, str(iface))
    rt = ipaddress.IPv4Interface("0.0.0.0/%s" % iface.netmask)
    assert str(rt) == base, ("netmask", i, str(rt))
print("ipv4_interface_prefix_netmask_roundtrip OK")
