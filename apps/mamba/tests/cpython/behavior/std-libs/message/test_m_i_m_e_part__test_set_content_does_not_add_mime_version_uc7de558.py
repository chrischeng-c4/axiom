# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "message"
# dimension = "behavior"
# case = "test_m_i_m_e_part__test_set_content_does_not_add_mime_version_uc7de558"
# subject = "cpython.test_message.TestMIMEPart.test_set_content_does_not_add_MIME_Version"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_message.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_message
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMIMEPart.test_set_content_does_not_add_MIME_Version", test_message)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMIMEPart.test_set_content_does_not_add_MIME_Version did not pass"
print("TestMIMEPart::test_set_content_does_not_add_MIME_Version: ok")
