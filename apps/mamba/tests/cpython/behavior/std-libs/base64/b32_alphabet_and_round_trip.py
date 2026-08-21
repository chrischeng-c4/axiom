# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b32_alphabet_and_round_trip"
# subject = "base64.b32encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b32encode: b32encode emits only the A-Z, 2-7, '=' alphabet and b32decode round-trips it back"""
import base64

_b32 = base64.b32encode(b"hello")
assert all(c in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567=" for c in _b32), _b32
assert base64.b32decode(_b32) == b"hello", "b32 round-trip"
print("b32_alphabet_and_round_trip OK")
