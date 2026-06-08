# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_hosts_excludes_endpoints"
# subject = "ipaddress.IPv4Network"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Network: hosts() of a /30 yields the 2 usable addresses, excluding the network and broadcast addresses"""
import ipaddress

net = ipaddress.IPv4Network("192.168.1.0/30")
hosts = list(net.hosts())
assert len(hosts) == 2, len(hosts)
assert str(hosts[0]) == "192.168.1.1", str(hosts[0])
assert str(hosts[1]) == "192.168.1.2", str(hosts[1])
print("network_hosts_excludes_endpoints OK")
