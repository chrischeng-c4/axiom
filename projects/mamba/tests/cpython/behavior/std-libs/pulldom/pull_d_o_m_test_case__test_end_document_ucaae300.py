# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pulldom"
# dimension = "behavior"
# case = "pull_d_o_m_test_case__test_end_document_ucaae300"
# subject = "cpython.test_pulldom.PullDOMTestCase.test_end_document"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pulldom.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pulldom
_suite = unittest.defaultTestLoader.loadTestsFromName("PullDOMTestCase.test_end_document", test_pulldom)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PullDOMTestCase.test_end_document did not pass"
print("PullDOMTestCase::test_end_document: ok")
