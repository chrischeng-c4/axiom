# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_x_y_test_case__test_b85encode_uc9a108b"
# subject = "cpython.test_base64.BaseXYTestCase.test_b85encode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_base64
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseXYTestCase.test_b85encode", test_base64)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseXYTestCase.test_b85encode did not pass"
print("BaseXYTestCase::test_b85encode: ok")
