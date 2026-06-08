# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "debugging_server_tests__testSendMessageWithSomeAddresses"
# subject = "cpython.test_smtplib.DebuggingServerTests.testSendMessageWithSomeAddresses"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtplib
_suite = unittest.defaultTestLoader.loadTestsFromName("DebuggingServerTests.testSendMessageWithSomeAddresses", test_smtplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DebuggingServerTests.testSendMessageWithSomeAddresses did not pass"
print("DebuggingServerTests::testSendMessageWithSomeAddresses: ok")
