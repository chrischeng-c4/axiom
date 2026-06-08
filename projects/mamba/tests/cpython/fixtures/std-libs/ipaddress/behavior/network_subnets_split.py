# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_subnets_split"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: subnets() splits a /23 into two /24 children"""
import ipaddress

parent = ipaddress.IPv4Network("192.168.0.0/23")
subnets = list(parent.subnets())
assert len(subnets) == 2, len(subnets)
assert subnets[0].prefixlen == 24, subnets[0].prefixlen
assert str(subnets[0]) == "192.168.0.0/24", str(subnets[0])
assert str(subnets[1]) == "192.168.1.0/24", str(subnets[1])
print("network_subnets_split OK")
