# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_version_and_packed_length"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: an IPv4 address reports version 4, a 4-byte packed form, and its compressed text equals the input"""
import ipaddress

a = ipaddress.ip_address("192.168.1.5")
assert a.version == 4, a.version
assert len(a.packed) == 4, len(a.packed)
assert a.compressed == "192.168.1.5", a.compressed
print("ipv4_version_and_packed_length OK")
