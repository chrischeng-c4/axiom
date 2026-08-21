# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_url_handler_test__test_url_requests_uc2dce26"
# subject = "cpython.test_pydoc.PydocUrlHandlerTest.test_url_requests"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("PydocUrlHandlerTest.test_url_requests", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PydocUrlHandlerTest.test_url_requests did not pass"
print("PydocUrlHandlerTest::test_url_requests: ok")
