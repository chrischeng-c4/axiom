# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_import_test__test_apropos_with_unreadable_dir_uc752115"
# subject = "cpython.test_pydoc.PydocImportTest.test_apropos_with_unreadable_dir"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("PydocImportTest.test_apropos_with_unreadable_dir", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PydocImportTest.test_apropos_with_unreadable_dir did not pass"
print("PydocImportTest::test_apropos_with_unreadable_dir: ok")
