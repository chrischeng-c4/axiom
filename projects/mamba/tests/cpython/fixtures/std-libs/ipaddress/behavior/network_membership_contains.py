# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_membership_contains"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: an address inside a /24 tests `in` True and an address outside tests `in` False"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/24")
assert ipaddress.ip_address("192.168.1.100") in net, "addr in network"
assert ipaddress.ip_address("10.0.0.1") not in net, "addr not in network"
print("network_membership_contains OK")
