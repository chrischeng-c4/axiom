# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_retrieving_source_code__test_getdoc"
# subject = "cpython.test_inspect.TestRetrievingSourceCode.test_getdoc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_inspect import test_inspect
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRetrievingSourceCode.test_getdoc", test_inspect)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRetrievingSourceCode.test_getdoc did not pass"
print("TestRetrievingSourceCode::test_getdoc: ok")
