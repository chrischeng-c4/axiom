# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_header__test_make_header_handles_binary_unknown8bit"
# subject = "cpython.test_email.TestHeader.test_make_header_handles_binary_unknown8bit"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestHeader.test_make_header_handles_binary_unknown8bit", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestHeader.test_make_header_handles_binary_unknown8bit did not pass"
print("TestHeader::test_make_header_handles_binary_unknown8bit: ok")
