# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "code_test_case__test_many_codeobjects_ucfebc59"
# subject = "cpython.test_marshal.CodeTestCase.test_many_codeobjects"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_marshal
_suite = unittest.defaultTestLoader.loadTestsFromName("CodeTestCase.test_many_codeobjects", test_marshal)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CodeTestCase.test_many_codeobjects did not pass"
print("CodeTestCase::test_many_codeobjects: ok")
