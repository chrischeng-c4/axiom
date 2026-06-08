# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "c_misc_i_o_test__test_readinto_buffer_overflow"
# subject = "cpython.test_io.CMiscIOTest.test_readinto_buffer_overflow"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_io
_suite = unittest.defaultTestLoader.loadTestsFromName("CMiscIOTest.test_readinto_buffer_overflow", test_io)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CMiscIOTest.test_readinto_buffer_overflow did not pass"
print("CMiscIOTest::test_readinto_buffer_overflow: ok")
