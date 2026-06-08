# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contentmanager"
# dimension = "behavior"
# case = "test_raw_data_manager__test_set_text_short_line_minimal_non_ascii_heuristics"
# subject = "cpython.test_contentmanager.TestRawDataManager.test_set_text_short_line_minimal_non_ascii_heuristics"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_contentmanager.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_contentmanager
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRawDataManager.test_set_text_short_line_minimal_non_ascii_heuristics", test_contentmanager)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRawDataManager.test_set_text_short_line_minimal_non_ascii_heuristics did not pass"
print("TestRawDataManager::test_set_text_short_line_minimal_non_ascii_heuristics: ok")
