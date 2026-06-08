# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ip_address_dispatches_ipv6"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: ip_address of a colon-hex string returns an IPv6Address whose str is the compressed form"""
import ipaddress

a = ipaddress.ip_address("fe80::1")
assert isinstance(a, ipaddress.IPv6Address), type(a)
assert str(ipaddress.ip_address("::1")) == "::1", "compressed loopback"
print("ip_address_dispatches_ipv6 OK")
