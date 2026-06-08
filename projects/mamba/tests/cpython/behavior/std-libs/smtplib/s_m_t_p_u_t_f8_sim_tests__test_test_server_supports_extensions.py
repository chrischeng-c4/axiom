# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "s_m_t_p_u_t_f8_sim_tests__test_test_server_supports_extensions"
# subject = "cpython.test_smtplib.SMTPUTF8SimTests.test_test_server_supports_extensions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtplib
_suite = unittest.defaultTestLoader.loadTestsFromName("SMTPUTF8SimTests.test_test_server_supports_extensions", test_smtplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SMTPUTF8SimTests.test_test_server_supports_extensions did not pass"
print("SMTPUTF8SimTests::test_test_server_supports_extensions: ok")
