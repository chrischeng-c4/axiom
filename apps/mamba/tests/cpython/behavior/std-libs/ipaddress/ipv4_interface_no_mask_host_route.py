# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_interface_no_mask_host_route"
# subject = "ipaddress.IPv4Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Interface: an IPv4Interface with no mask (from str/int/bytes) is a /32 with all-ones netmask and zero hostmask"""
import ipaddress

for addr in ("1.2.3.4", 16909060, b"\x01\x02\x03\x04"):
    iface = ipaddress.IPv4Interface(addr)
    assert str(iface) == "1.2.3.4/32", str(iface)
    assert str(iface.netmask) == "255.255.255.255", "netmask"
    assert str(iface.hostmask) == "0.0.0.0", "hostmask"
print("ipv4_interface_no_mask_host_route OK")
