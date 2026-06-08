# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "as_parameter"
# dimension = "behavior"
# case = "basic_wrap_test_case__test_byval_ucb1eb17"
# subject = "cpython.test_as_parameter.BasicWrapTestCase.test_byval"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_as_parameter.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_as_parameter
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicWrapTestCase.test_byval", test_as_parameter)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicWrapTestCase.test_byval did not pass"
print("BasicWrapTestCase::test_byval: ok")
