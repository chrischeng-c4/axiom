# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hex_roundtrip"
# subject = "binascii.unhexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.unhexlify: unhexlify inverts hexlify over arbitrary bytes (round-trip)"""
import binascii

_data = b"hello world"
_hex = binascii.hexlify(_data)
assert isinstance(_hex, bytes), f"hexlify type = {type(_hex)!r}"
_raw = binascii.unhexlify(_hex)
assert isinstance(_raw, bytes), f"unhexlify type = {type(_raw)!r}"
assert _raw == _data, f"hex round-trip = {_raw!r}"
assert binascii.unhexlify(b"0102ff") == b"\x01\x02\xff", "unhexlify 0102ff"

print("hex_roundtrip OK")
