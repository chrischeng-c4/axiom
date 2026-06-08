# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpnet"
# dimension = "behavior"
# case = "smtp_s_s_l_test__test_connect_default_port_uc1bfbe0"
# subject = "cpython.test_smtpnet.SmtpSSLTest.test_connect_default_port"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtpnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtpnet
_suite = unittest.defaultTestLoader.loadTestsFromName("SmtpSSLTest.test_connect_default_port", test_smtpnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SmtpSSLTest.test_connect_default_port did not pass"
print("SmtpSSLTest::test_connect_default_port: ok")
