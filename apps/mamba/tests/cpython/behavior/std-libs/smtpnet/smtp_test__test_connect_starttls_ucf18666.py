# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpnet"
# dimension = "behavior"
# case = "smtp_test__test_connect_starttls_ucf18666"
# subject = "cpython.test_smtpnet.SmtpTest.test_connect_starttls"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtpnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtpnet
_suite = unittest.defaultTestLoader.loadTestsFromName("SmtpTest.test_connect_starttls", test_smtpnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SmtpTest.test_connect_starttls did not pass"
print("SmtpTest::test_connect_starttls: ok")
