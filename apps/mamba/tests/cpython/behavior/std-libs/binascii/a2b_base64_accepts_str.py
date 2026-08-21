# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_base64_accepts_str"
# subject = "binascii.a2b_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: a2b_base64 accepts ASCII str input, not just bytes"""
import binascii

assert binascii.a2b_base64("aGVsbG8=") == b"hello", "a2b_base64 str input"
assert binascii.a2b_base64(b"aGVsbG8=") == b"hello", "a2b_base64 bytes input"

print("a2b_base64_accepts_str OK")
