# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "s_m_t_p_sim_tests__testAUTH_LOGIN_initial_response_ok"
# subject = "cpython.test_smtplib.SMTPSimTests.testAUTH_LOGIN_initial_response_ok"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtplib
_suite = unittest.defaultTestLoader.loadTestsFromName("SMTPSimTests.testAUTH_LOGIN_initial_response_ok", test_smtplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SMTPSimTests.testAUTH_LOGIN_initial_response_ok did not pass"
print("SMTPSimTests::testAUTH_LOGIN_initial_response_ok: ok")
