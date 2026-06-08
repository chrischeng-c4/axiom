# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_mapped_ipv6_extraction"
# subject = "ipaddress.IPv6Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv6Address: an IPv4-mapped IPv6 address exposes the embedded IPv4Address via .ipv4_mapped"""
import ipaddress

a = ipaddress.IPv6Address("::ffff:192.168.1.1")
assert a.ipv4_mapped == ipaddress.IPv4Address("192.168.1.1"), a.ipv4_mapped
print("ipv4_mapped_ipv6_extraction OK")
