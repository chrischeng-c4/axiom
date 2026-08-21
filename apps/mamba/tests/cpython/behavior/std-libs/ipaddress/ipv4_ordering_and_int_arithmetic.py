# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_ordering_and_int_arithmetic"
# subject = "ipaddress.IPv4Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv4Address: adjacent IPv4 addresses order correctly and differ by 1 under int() conversion and addition"""
import ipaddress

a1 = ipaddress.IPv4Address("192.168.1.1")
a2 = ipaddress.IPv4Address("192.168.1.2")
assert a1 < a2, "ordering"
assert int(a2) - int(a1) == 1, int(a2) - int(a1)
assert int(a1) + 1 == int(a2), "int arithmetic"
print("ipv4_ordering_and_int_arithmetic OK")
