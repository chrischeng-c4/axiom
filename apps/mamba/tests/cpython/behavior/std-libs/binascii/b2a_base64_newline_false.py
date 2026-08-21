# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_base64_newline_false"
# subject = "binascii.b2a_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_base64: b2a_base64(newline=False) drops the trailing newline"""
import binascii

_b64n = binascii.b2a_base64(b"abc", newline=False)
assert not _b64n.endswith(b"\n"), f"no newline = {_b64n!r}"
assert _b64n == b"YWJj", f"b2a_base64 no-newline = {_b64n!r}"

print("b2a_base64_newline_false OK")
