# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_version_and_packed_length"
# subject = "ipaddress.ip_address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.ip_address: an IPv6 address reports version 6, a 16-byte packed form, and compresses ::1 correctly"""
import ipaddress

a = ipaddress.ip_address("::1")
assert a.version == 6, a.version
assert len(a.packed) == 16, len(a.packed)
assert a.compressed == "::1", a.compressed
print("ipv6_version_and_packed_length OK")
