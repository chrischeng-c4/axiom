# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "exception_test_case__test_disallow_instantiation_ucbf06b3"
# subject = "cpython.test_zlib.ExceptionTestCase.test_disallow_instantiation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionTestCase.test_disallow_instantiation", test_zlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionTestCase.test_disallow_instantiation did not pass"
print("ExceptionTestCase::test_disallow_instantiation: ok")
