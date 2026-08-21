# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "byteswap"
# dimension = "behavior"
# case = "test__test_endian_other_uc6359cd"
# subject = "cpython.test_byteswap.Test.test_endian_other"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_byteswap.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from binascii import hexlify
from ctypes import *
assert c_byte.__ctype_le__ is c_byte
assert c_byte.__ctype_be__ is c_byte
assert c_ubyte.__ctype_le__ is c_ubyte
assert c_ubyte.__ctype_be__ is c_ubyte
assert c_char.__ctype_le__ is c_char
assert c_char.__ctype_be__ is c_char

print("Test::test_endian_other: ok")
