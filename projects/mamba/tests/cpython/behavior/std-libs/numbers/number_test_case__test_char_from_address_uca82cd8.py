# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "number_test_case__test_char_from_address_uca82cd8"
# subject = "cpython.test_numbers.NumberTestCase.test_char_from_address"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import struct
from ctypes import c_char
from array import array
a = array('b', [0])
a[0] = ord('x')
v = c_char.from_address(a.buffer_info()[0])
assert v.value == b'x'
assert type(v) is c_char
a[0] = ord('?')
assert v.value == b'?'

print("NumberTestCase::test_char_from_address: ok")
