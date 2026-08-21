# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "base64_roundtrip"
# subject = "binascii.a2b_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64 inverts b2a_base64 over arbitrary bytes (round-trip)"""
import binascii

_data = bytes(range(0, 256, 3))
_encoded = binascii.b2a_base64(_data, newline=False)
assert binascii.a2b_base64(_encoded) == _data, "base64 round-trip"

print("base64_roundtrip OK")
