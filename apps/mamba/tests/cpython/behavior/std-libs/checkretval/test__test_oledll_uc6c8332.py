# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "checkretval"
# dimension = "behavior"
# case = "test__test_oledll_uc6c8332"
# subject = "cpython.test_checkretval.Test.test_oledll"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_checkretval.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_checkretval
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_oledll", test_checkretval)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_oledll did not pass"
print("Test::test_oledll: ok")
