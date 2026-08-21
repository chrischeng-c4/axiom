# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "headerregistry"
# dimension = "behavior"
# case = "test_folding__test_unstructured_short_max_line_length"
# subject = "cpython.test_headerregistry.TestFolding.test_unstructured_short_max_line_length"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_headerregistry.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_headerregistry
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFolding.test_unstructured_short_max_line_length", test_headerregistry)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFolding.test_unstructured_short_max_line_length did not pass"
print("TestFolding::test_unstructured_short_max_line_length: ok")
