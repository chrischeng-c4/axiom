# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_doc_test__test_mixed_case_module_names_are_lower_cased_uc4ef7f7"
# subject = "cpython.test_pydoc.PydocDocTest.test_mixed_case_module_names_are_lower_cased"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("PydocDocTest.test_mixed_case_module_names_are_lower_cased", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PydocDocTest.test_mixed_case_module_names_are_lower_cased did not pass"
print("PydocDocTest::test_mixed_case_module_names_are_lower_cased: ok")
