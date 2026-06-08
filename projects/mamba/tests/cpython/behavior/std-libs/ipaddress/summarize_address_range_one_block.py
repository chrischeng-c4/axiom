# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "summarize_address_range_one_block"
# subject = "ipaddress.summarize_address_range"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.summarize_address_range: summarize_address_range over 192.168.1.0..255 yields the single block 192.168.1.0/24"""
import ipaddress

summary = list(ipaddress.summarize_address_range(
    ipaddress.IPv4Address("192.168.1.0"),
    ipaddress.IPv4Address("192.168.1.255"),
))
assert len(summary) == 1, len(summary)
assert str(summary[0]) == "192.168.1.0/24", str(summary[0])
print("summarize_address_range_one_block OK")
