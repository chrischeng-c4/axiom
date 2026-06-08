# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contentmanager"
# dimension = "behavior"
# case = "test_content_manager__test_get_content_raises_if_unknown_mimetype_and_no_default"
# subject = "cpython.test_contentmanager.TestContentManager.test_get_content_raises_if_unknown_mimetype_and_no_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_contentmanager.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_contentmanager
_suite = unittest.defaultTestLoader.loadTestsFromName("TestContentManager.test_get_content_raises_if_unknown_mimetype_and_no_default", test_contentmanager)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestContentManager.test_get_content_raises_if_unknown_mimetype_and_no_default did not pass"
print("TestContentManager::test_get_content_raises_if_unknown_mimetype_and_no_default: ok")
