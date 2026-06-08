# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_x_y_test_case__test_b32hexencode_other_types_uc04ec6e"
# subject = "cpython.test_base64.BaseXYTestCase.test_b32hexencode_other_types"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_base64
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseXYTestCase.test_b32hexencode_other_types", test_base64)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseXYTestCase.test_b32hexencode_other_types did not pass"
print("BaseXYTestCase::test_b32hexencode_other_types: ok")
