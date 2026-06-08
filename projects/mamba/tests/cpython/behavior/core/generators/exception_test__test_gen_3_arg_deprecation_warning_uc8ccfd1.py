# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "exception_test__test_gen_3_arg_deprecation_warning_uc8ccfd1"
# subject = "cpython.test_generators.ExceptionTest.test_gen_3_arg_deprecation_warning"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_generators.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_generators
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionTest.test_gen_3_arg_deprecation_warning", test_generators)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionTest.test_gen_3_arg_deprecation_warning did not pass"
print("ExceptionTest::test_gen_3_arg_deprecation_warning: ok")
