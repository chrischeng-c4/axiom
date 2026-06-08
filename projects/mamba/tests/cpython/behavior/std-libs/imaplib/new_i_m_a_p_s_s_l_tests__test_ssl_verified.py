# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "behavior"
# case = "new_i_m_a_p_s_s_l_tests__test_ssl_verified"
# subject = "cpython.test_imaplib.NewIMAPSSLTests.test_ssl_verified"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imaplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_imaplib
_suite = unittest.defaultTestLoader.loadTestsFromName("NewIMAPSSLTests.test_ssl_verified", test_imaplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NewIMAPSSLTests.test_ssl_verified did not pass"
print("NewIMAPSSLTests::test_ssl_verified: ok")
