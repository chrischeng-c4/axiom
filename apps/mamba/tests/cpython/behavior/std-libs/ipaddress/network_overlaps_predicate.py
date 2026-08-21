# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_overlaps_predicate"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: overlaps() is True for a network and its enclosed half and False for a disjoint network"""
import ipaddress

n1 = ipaddress.IPv4Network("192.168.1.0/24")
n2 = ipaddress.IPv4Network("192.168.1.128/25")
n3 = ipaddress.IPv4Network("10.0.0.0/8")
assert n1.overlaps(n2), "overlapping networks"
assert not n1.overlaps(n3), "non-overlapping networks"
print("network_overlaps_predicate OK")
