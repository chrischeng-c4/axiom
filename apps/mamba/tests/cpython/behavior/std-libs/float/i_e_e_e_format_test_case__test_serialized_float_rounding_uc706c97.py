# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "float"
# dimension = "behavior"
# case = "i_e_e_e_format_test_case__test_serialized_float_rounding_uc706c97"
# subject = "cpython.test_float.IEEEFormatTestCase.test_serialized_float_rounding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_float
_suite = unittest.defaultTestLoader.loadTestsFromName("IEEEFormatTestCase.test_serialized_float_rounding", test_float)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IEEEFormatTestCase.test_serialized_float_rounding did not pass"
print("IEEEFormatTestCase::test_serialized_float_rounding: ok")
