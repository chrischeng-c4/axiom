# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "code_test_case__test_code_uc415df8"
# subject = "cpython.test_marshal.CodeTestCase.test_code"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_marshal
_suite = unittest.defaultTestLoader.loadTestsFromName("CodeTestCase.test_code", test_marshal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CodeTestCase.test_code did not pass"
print("CodeTestCase::test_code: ok")
