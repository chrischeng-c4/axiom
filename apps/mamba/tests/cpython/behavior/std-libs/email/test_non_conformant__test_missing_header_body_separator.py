# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_non_conformant__test_missing_header_body_separator"
# subject = "cpython.test_email.TestNonConformant.test_missing_header_body_separator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNonConformant.test_missing_header_body_separator", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNonConformant.test_missing_header_body_separator did not pass"
print("TestNonConformant::test_missing_header_body_separator: ok")
