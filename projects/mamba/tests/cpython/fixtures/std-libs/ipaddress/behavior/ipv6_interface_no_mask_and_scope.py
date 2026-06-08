# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_interface_no_mask_and_scope"
# subject = "ipaddress.IPv6Interface"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Interface: an IPv6Interface defaults to /128 with all-ones netmask, and scope identifiers survive in the interface text"""
import ipaddress

for addr in ("::1", 1, b"\x00" * 15 + b"\x01"):
    iface = ipaddress.IPv6Interface(addr)
    assert str(iface) == "::1/128", str(iface)
    assert str(iface.netmask) == "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", "netmask"
    assert str(iface.hostmask) == "::", "hostmask"
scoped = ipaddress.IPv6Interface("::1%scope")
assert str(scoped) == "::1%scope/128", str(scoped)
print("ipv6_interface_no_mask_and_scope OK")
