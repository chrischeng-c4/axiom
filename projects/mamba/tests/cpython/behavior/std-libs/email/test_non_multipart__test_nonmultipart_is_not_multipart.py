# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_non_multipart__test_nonmultipart_is_not_multipart"
# subject = "cpython.test_email.TestNonMultipart.test_nonmultipart_is_not_multipart"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNonMultipart.test_nonmultipart_is_not_multipart", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNonMultipart.test_nonmultipart_is_not_multipart did not pass"
print("TestNonMultipart::test_nonmultipart_is_not_multipart: ok")
