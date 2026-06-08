# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "collapse_addresses_merges_adjacent"
# subject = "ipaddress.collapse_addresses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.collapse_addresses: collapse_addresses merges two adjacent /25s into one /24"""
import ipaddress

addrs = [
    ipaddress.IPv4Network("192.168.1.0/25"),
    ipaddress.IPv4Network("192.168.1.128/25"),
]
collapsed = list(ipaddress.collapse_addresses(addrs))
assert len(collapsed) == 1, len(collapsed)
assert str(collapsed[0]) == "192.168.1.0/24", str(collapsed[0])
print("collapse_addresses_merges_adjacent OK")
