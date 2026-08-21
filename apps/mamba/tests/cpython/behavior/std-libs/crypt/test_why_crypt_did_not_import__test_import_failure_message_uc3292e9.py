# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "behavior"
# case = "test_why_crypt_did_not_import__test_import_failure_message_uc3292e9"
# subject = "cpython.test_crypt.TestWhyCryptDidNotImport.test_import_failure_message"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_crypt.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_crypt
_suite = unittest.defaultTestLoader.loadTestsFromName("TestWhyCryptDidNotImport.test_import_failure_message", test_crypt)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestWhyCryptDidNotImport.test_import_failure_message did not pass"
print("TestWhyCryptDidNotImport::test_import_failure_message: ok")
