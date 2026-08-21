# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_uu_known_encodings"
# subject = "binascii.b2a_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: b2a_uu known encodings for single byte, empty, and a NUL-prefixed block"""
import binascii

assert binascii.b2a_uu(b"x") == b"!>   \n", "b2a_uu('x')"
assert binascii.b2a_uu(b"") == b" \n", "b2a_uu(empty)"
assert binascii.b2a_uu(b"\x00Cat") == b"$ $-A=   \n", "b2a_uu('\\x00Cat')"

print("b2a_uu_known_encodings OK")
