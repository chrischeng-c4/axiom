# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "repr"
# dimension = "behavior"
# case = "repr_test__test_char_uc00e93c"
# subject = "cpython.test_repr.ReprTest.test_char"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_repr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_repr
_suite = unittest.defaultTestLoader.loadTestsFromName("ReprTest.test_char", test_repr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReprTest.test_char did not pass"
print("ReprTest::test_char: ok")
