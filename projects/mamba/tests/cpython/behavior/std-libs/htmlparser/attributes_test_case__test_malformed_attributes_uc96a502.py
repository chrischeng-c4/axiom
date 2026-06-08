# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "htmlparser"
# dimension = "behavior"
# case = "attributes_test_case__test_malformed_attributes_uc96a502"
# subject = "cpython.test_htmlparser.AttributesTestCase.test_malformed_attributes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_htmlparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_htmlparser
_suite = unittest.defaultTestLoader.loadTestsFromName("AttributesTestCase.test_malformed_attributes", test_htmlparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AttributesTestCase.test_malformed_attributes did not pass"
print("AttributesTestCase::test_malformed_attributes: ok")
