# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_base64_newline_default"
# subject = "binascii.b2a_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_base64: b2a_base64 appends a trailing newline by default (newline=True)"""
import binascii

_b64 = binascii.b2a_base64(b"abc")
assert isinstance(_b64, bytes), f"b2a_base64 type = {type(_b64)!r}"
assert _b64.endswith(b"\n"), f"b2a_base64 newline = {_b64!r}"
assert binascii.b2a_base64(b"abc", newline=True) == _b64, "newline=True is the default"
assert binascii.b2a_base64(b"hello") == b"aGVsbG8=\n", "known encoding with newline"

print("b2a_base64_newline_default OK")
