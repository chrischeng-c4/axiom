# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "uu_roundtrip"
# subject = "binascii.b2a_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: a2b_uu inverts b2a_uu over a 45-byte block (per-line maximum)"""
import binascii

_raw = b"The quick brown fox jumps over the lazy dog.."[:45]
_line = binascii.b2a_uu(_raw)
assert binascii.a2b_uu(_line) == _raw, f"uu round-trip = {_line!r}"

print("uu_roundtrip OK")
