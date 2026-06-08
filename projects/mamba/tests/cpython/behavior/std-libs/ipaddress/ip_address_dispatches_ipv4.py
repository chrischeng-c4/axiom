# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ip_address_dispatches_ipv4"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: ip_address of a dotted-quad string returns an IPv4Address whose str, packed, and int forms match the input"""
import ipaddress

a = ipaddress.ip_address("192.168.1.1")
assert isinstance(a, ipaddress.IPv4Address), type(a)
assert str(a) == "192.168.1.1", str(a)
assert a.packed == b"\xc0\xa8\x01\x01", a.packed
assert int(a) == 0xC0A80101, int(a)
print("ip_address_dispatches_ipv4 OK")
