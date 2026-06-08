# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_address_netmask_count"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: a /24 reports prefixlen 24, network_address, dotted netmask, and 256 num_addresses"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/24")
assert net.prefixlen == 24, net.prefixlen
assert str(net.network_address) == "192.168.1.0", str(net.network_address)
assert str(net.netmask) == "255.255.255.0", str(net.netmask)
assert net.num_addresses == 256, net.num_addresses
print("network_address_netmask_count OK")
