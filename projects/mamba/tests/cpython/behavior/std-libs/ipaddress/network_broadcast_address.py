# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_broadcast_address"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: the broadcast_address of 192.168.1.0/24 is 192.168.1.255"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/24")
assert str(net.broadcast_address) == "192.168.1.255", str(net.broadcast_address)
print("network_broadcast_address OK")
