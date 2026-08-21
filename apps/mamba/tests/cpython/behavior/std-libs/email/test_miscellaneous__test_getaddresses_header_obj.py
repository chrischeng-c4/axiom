# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_miscellaneous__test_getaddresses_header_obj"
# subject = "cpython.test_email.TestMiscellaneous.test_getaddresses_header_obj"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMiscellaneous.test_getaddresses_header_obj", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMiscellaneous.test_getaddresses_header_obj did not pass"
print("TestMiscellaneous::test_getaddresses_header_obj: ok")
