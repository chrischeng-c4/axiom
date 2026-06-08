# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "exception_test__test_except_throw_bad_exception_uc7866b0"
# subject = "cpython.test_generators.ExceptionTest.test_except_throw_bad_exception"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_generators.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_generators
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionTest.test_except_throw_bad_exception", test_generators)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionTest.test_except_throw_bad_exception did not pass"
print("ExceptionTest::test_except_throw_bad_exception: ok")
