# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bitfields"
# dimension = "behavior"
# case = "bit_field_test__test_uint32_swap_little_endian_ucfe58ea"
# subject = "cpython.test_bitfields.BitFieldTest.test_uint32_swap_little_endian"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_bitfields.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_bitfields
_suite = unittest.defaultTestLoader.loadTestsFromName("BitFieldTest.test_uint32_swap_little_endian", test_bitfields)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BitFieldTest.test_uint32_swap_little_endian did not pass"
print("BitFieldTest::test_uint32_swap_little_endian: ok")
