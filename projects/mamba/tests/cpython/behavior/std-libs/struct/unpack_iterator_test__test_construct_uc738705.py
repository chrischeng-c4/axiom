# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_iterator_test__test_construct_uc738705"
# subject = "cpython.test_struct.UnpackIteratorTest.test_construct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_struct
_suite = unittest.defaultTestLoader.loadTestsFromName("UnpackIteratorTest.test_construct", test_struct)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnpackIteratorTest.test_construct did not pass"
print("UnpackIteratorTest::test_construct: ok")
