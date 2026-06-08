# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "v4_int_to_packed_big_endian"
# subject = "ipaddress.v4_int_to_packed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.v4_int_to_packed: v4_int_to_packed(0xC0A80101) is the 4-byte big-endian b'\\xc0\\xa8\\x01\\x01'"""
import ipaddress

pkt = ipaddress.v4_int_to_packed(0xC0A80101)
assert pkt == b"\xc0\xa8\x01\x01", pkt
assert len(pkt) == 4, len(pkt)
assert (pkt[0], pkt[1], pkt[2], pkt[3]) == (192, 168, 1, 1), tuple(pkt)
print("v4_int_to_packed_big_endian OK")
