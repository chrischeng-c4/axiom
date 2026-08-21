# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_pointers_bool_uc1aacda"
# subject = "cpython.test_pointers.PointersTestCase.test_pointers_bool"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_pointers
_suite = unittest.defaultTestLoader.loadTestsFromName("PointersTestCase.test_pointers_bool", test_pointers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PointersTestCase.test_pointers_bool did not pass"
print("PointersTestCase::test_pointers_bool: ok")
