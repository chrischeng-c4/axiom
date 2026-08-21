# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv6_format_spec_variants"
# subject = "ipaddress.IPv6Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv6Address: format() on an IPv6Address packs to 32 nibbles for hex specs, groups with '_', and uses the compressed text for 's'/''; binary form is 128 bits wide"""
import ipaddress

v6 = ipaddress.IPv6Address("::1.2.3.42")
assert format(v6, "x") == "0000000000000000000000000102032a", "v6 x"
assert format(v6, "X") == "0000000000000000000000000102032A", "v6 X"
assert format(v6, "_x") == "0000_0000_0000_0000_0000_0000_0102_032a", "v6 _x"
assert format(v6, "#x") == "0x0000000000000000000000000102032a", "v6 #x"
assert format(v6, "s") == "::102:32a", "v6 s"
assert format(v6, "") == "::102:32a", "v6 default"
assert len(format(v6, "b")) == 128, "v6 binary width"
print("ipv6_format_spec_variants OK")
