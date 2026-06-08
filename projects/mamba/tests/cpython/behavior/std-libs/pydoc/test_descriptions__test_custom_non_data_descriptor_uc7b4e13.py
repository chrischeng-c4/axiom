# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "test_descriptions__test_custom_non_data_descriptor_uc7b4e13"
# subject = "cpython.test_pydoc.TestDescriptions.test_custom_non_data_descriptor"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDescriptions.test_custom_non_data_descriptor", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDescriptions.test_custom_non_data_descriptor did not pass"
print("TestDescriptions::test_custom_non_data_descriptor: ok")
