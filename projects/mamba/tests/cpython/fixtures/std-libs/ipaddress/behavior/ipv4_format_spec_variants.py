# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "ipv4_format_spec_variants"
# subject = "ipaddress.IPv4Address"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
"""ipaddress.IPv4Address: format() on an IPv4Address honors b/x/X, grouping '_', the alternate '#' prefix, 'n' (== 'b'), and 's'/'' textual forms"""
import ipaddress

v4 = ipaddress.IPv4Address("1.2.3.42")
cases = {
    "b": "00000001000000100000001100101010",
    "x": "0102032a",
    "X": "0102032A",
    "_x": "0102_032a",
    "#x": "0x0102032a",
    "#X": "0X0102032A",
    "#_X": "0X0102_032A",
    "s": "1.2.3.42",
    "": "1.2.3.42",
}
for spec, want in cases.items():
    got = format(v4, spec)
    assert got == want, (spec, got, want)
assert format(v4, "n") == format(v4, "b"), "v4 n == b"
print("ipv4_format_spec_variants OK")
