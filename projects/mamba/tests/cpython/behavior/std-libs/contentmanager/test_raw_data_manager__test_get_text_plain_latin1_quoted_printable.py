# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contentmanager"
# dimension = "behavior"
# case = "test_raw_data_manager__test_get_text_plain_latin1_quoted_printable"
# subject = "cpython.test_contentmanager.TestRawDataManager.test_get_text_plain_latin1_quoted_printable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_contentmanager.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_contentmanager
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRawDataManager.test_get_text_plain_latin1_quoted_printable", test_contentmanager)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRawDataManager.test_get_text_plain_latin1_quoted_printable did not pass"
print("TestRawDataManager::test_get_text_plain_latin1_quoted_printable: ok")
