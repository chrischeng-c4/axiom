# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "behavior"
# case = "remote_i_m_a_p__s_s_l_test__test_logincapa"
# subject = "cpython.test_imaplib.RemoteIMAP_SSLTest.test_logincapa"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imaplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_imaplib
_suite = unittest.defaultTestLoader.loadTestsFromName("RemoteIMAP_SSLTest.test_logincapa", test_imaplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RemoteIMAP_SSLTest.test_logincapa did not pass"
print("RemoteIMAP_SSLTest::test_logincapa: ok")
