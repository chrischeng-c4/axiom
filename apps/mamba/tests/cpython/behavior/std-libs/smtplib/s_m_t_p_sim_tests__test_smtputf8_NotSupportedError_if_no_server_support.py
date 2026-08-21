# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "s_m_t_p_sim_tests__test_smtputf8_NotSupportedError_if_no_server_support"
# subject = "cpython.test_smtplib.SMTPSimTests.test_smtputf8_NotSupportedError_if_no_server_support"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtplib
_suite = unittest.defaultTestLoader.loadTestsFromName("SMTPSimTests.test_smtputf8_NotSupportedError_if_no_server_support", test_smtplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SMTPSimTests.test_smtputf8_NotSupportedError_if_no_server_support did not pass"
print("SMTPSimTests::test_smtputf8_NotSupportedError_if_no_server_support: ok")
