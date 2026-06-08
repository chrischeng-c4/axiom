# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailcap"
# dimension = "behavior"
# case = "helper_function_test__test_readmailcapfile_uc18397c"
# subject = "cpython.test_mailcap.HelperFunctionTest.test_readmailcapfile"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailcap.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mailcap
_suite = unittest.defaultTestLoader.loadTestsFromName("HelperFunctionTest.test_readmailcapfile", test_mailcap)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HelperFunctionTest.test_readmailcapfile did not pass"
print("HelperFunctionTest::test_readmailcapfile: ok")
