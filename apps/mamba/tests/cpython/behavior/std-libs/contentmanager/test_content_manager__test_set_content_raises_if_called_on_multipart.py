# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contentmanager"
# dimension = "behavior"
# case = "test_content_manager__test_set_content_raises_if_called_on_multipart"
# subject = "cpython.test_contentmanager.TestContentManager.test_set_content_raises_if_called_on_multipart"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_contentmanager.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_contentmanager
_suite = unittest.defaultTestLoader.loadTestsFromName("TestContentManager.test_set_content_raises_if_called_on_multipart", test_contentmanager)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestContentManager.test_set_content_raises_if_called_on_multipart did not pass"
print("TestContentManager::test_set_content_raises_if_called_on_multipart: ok")
