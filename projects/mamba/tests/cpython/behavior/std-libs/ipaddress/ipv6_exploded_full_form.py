# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_exploded_full_form"
# subject = "ipaddress.IPv6Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.IPv6Address: IPv6Address('::1').exploded is the fully zero-padded eight-group form"""
import ipaddress

a = ipaddress.IPv6Address("::1")
assert a.exploded == "0000:0000:0000:0000:0000:0000:0000:0001", a.exploded
print("ipv6_exploded_full_form OK")
