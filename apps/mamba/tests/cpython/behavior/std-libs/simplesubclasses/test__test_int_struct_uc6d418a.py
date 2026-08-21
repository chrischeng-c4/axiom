# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "simplesubclasses"
# dimension = "behavior"
# case = "test__test_int_struct_uc6d418a"
# subject = "cpython.test_simplesubclasses.Test.test_int_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_simplesubclasses.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_simplesubclasses
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_int_struct", test_simplesubclasses)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_int_struct did not pass"
print("Test::test_int_struct: ok")
