# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_stringification_uce03ba3"
# subject = "cpython.test_string.TestTemplate.test_stringification"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_string
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTemplate.test_stringification", test_string)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTemplate.test_stringification did not pass"
print("TestTemplate::test_stringification: ok")
