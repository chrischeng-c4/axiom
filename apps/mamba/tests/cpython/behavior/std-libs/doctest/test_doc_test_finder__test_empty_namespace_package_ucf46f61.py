# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "behavior"
# case = "test_doc_test_finder__test_empty_namespace_package_ucf46f61"
# subject = "cpython.test_doctest.TestDocTestFinder.test_empty_namespace_package"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_doctest/test_doctest.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_doctest import test_doctest
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDocTestFinder.test_empty_namespace_package", test_doctest)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDocTestFinder.test_empty_namespace_package did not pass"
print("TestDocTestFinder::test_empty_namespace_package: ok")
