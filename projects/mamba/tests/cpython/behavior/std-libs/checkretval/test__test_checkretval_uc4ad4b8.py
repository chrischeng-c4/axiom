# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "checkretval"
# dimension = "behavior"
# case = "test__test_checkretval_uc4ad4b8"
# subject = "cpython.test_checkretval.Test.test_checkretval"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_checkretval.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_checkretval
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_checkretval", test_checkretval)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_checkretval did not pass"
print("Test::test_checkretval: ok")
