# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_from_address_ucc73c40"
# subject = "cpython.test_arrays.ArrayTestCase.test_from_address"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *
p = create_string_buffer(b'foo')
sz = (c_char * 3).from_address(addressof(p))
assert sz[:] == b'foo'
assert sz[:] == b'foo'
assert sz[::-1] == b'oof'
assert sz[::3] == b'f'
assert sz[1:4:2] == b'o'
assert sz.value == b'foo'

print("ArrayTestCase::test_from_address: ok")
