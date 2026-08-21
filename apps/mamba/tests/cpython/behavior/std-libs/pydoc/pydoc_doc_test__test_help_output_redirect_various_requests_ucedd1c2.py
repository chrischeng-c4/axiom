# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_doc_test__test_help_output_redirect_various_requests_ucedd1c2"
# subject = "cpython.test_pydoc.PydocDocTest.test_help_output_redirect_various_requests"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("PydocDocTest.test_help_output_redirect_various_requests", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PydocDocTest.test_help_output_redirect_various_requests did not pass"
print("PydocDocTest::test_help_output_redirect_various_requests: ok")
