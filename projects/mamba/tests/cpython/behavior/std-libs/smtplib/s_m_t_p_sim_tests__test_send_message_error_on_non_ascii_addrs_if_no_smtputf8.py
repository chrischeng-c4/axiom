# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "s_m_t_p_sim_tests__test_send_message_error_on_non_ascii_addrs_if_no_smtputf8"
# subject = "cpython.test_smtplib.SMTPSimTests.test_send_message_error_on_non_ascii_addrs_if_no_smtputf8"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtplib
_suite = unittest.defaultTestLoader.loadTestsFromName("SMTPSimTests.test_send_message_error_on_non_ascii_addrs_if_no_smtputf8", test_smtplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SMTPSimTests.test_send_message_error_on_non_ascii_addrs_if_no_smtputf8 did not pass"
print("SMTPSimTests::test_send_message_error_on_non_ascii_addrs_if_no_smtputf8: ok")
