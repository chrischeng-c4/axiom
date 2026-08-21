# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_issue31311_uc76ad63"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_issue31311"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_parameters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_parameters
_suite = unittest.defaultTestLoader.loadTestsFromName("SimpleTypesTestCase.test_issue31311", test_parameters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimpleTypesTestCase.test_issue31311 did not pass"
print("SimpleTypesTestCase::test_issue31311: ok")
