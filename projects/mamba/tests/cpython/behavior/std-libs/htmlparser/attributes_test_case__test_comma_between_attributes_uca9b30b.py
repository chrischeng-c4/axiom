# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "htmlparser"
# dimension = "behavior"
# case = "attributes_test_case__test_comma_between_attributes_uca9b30b"
# subject = "cpython.test_htmlparser.AttributesTestCase.test_comma_between_attributes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_htmlparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_htmlparser
_suite = unittest.defaultTestLoader.loadTestsFromName("AttributesTestCase.test_comma_between_attributes", test_htmlparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AttributesTestCase.test_comma_between_attributes did not pass"
print("AttributesTestCase::test_comma_between_attributes: ok")
