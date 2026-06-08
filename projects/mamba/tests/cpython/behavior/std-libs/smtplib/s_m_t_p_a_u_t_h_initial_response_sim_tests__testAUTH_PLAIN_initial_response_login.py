# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "s_m_t_p_a_u_t_h_initial_response_sim_tests__testAUTH_PLAIN_initial_response_login"
# subject = "cpython.test_smtplib.SMTPAUTHInitialResponseSimTests.testAUTH_PLAIN_initial_response_login"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtplib
_suite = unittest.defaultTestLoader.loadTestsFromName("SMTPAUTHInitialResponseSimTests.testAUTH_PLAIN_initial_response_login", test_smtplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SMTPAUTHInitialResponseSimTests.testAUTH_PLAIN_initial_response_login did not pass"
print("SMTPAUTHInitialResponseSimTests::testAUTH_PLAIN_initial_response_login: ok")
