# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_base64_ignores_whitespace"
# subject = "binascii.a2b_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64 skips embedded whitespace/newlines while decoding"""
import binascii

assert binascii.a2b_base64(b"aGVs\nbG8=") == b"hello", "a2b with embedded newline"
assert binascii.a2b_base64(b"aGVsbG8=") == b"hello", "a2b without whitespace"

print("a2b_base64_ignores_whitespace OK")
