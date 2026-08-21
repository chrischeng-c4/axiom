# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "behavior"
# case = "remote_i_m_a_p__s_t_a_r_t_t_l_s_test__test_logincapa"
# subject = "cpython.test_imaplib.RemoteIMAP_STARTTLSTest.test_logincapa"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imaplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_imaplib
_suite = unittest.defaultTestLoader.loadTestsFromName("RemoteIMAP_STARTTLSTest.test_logincapa", test_imaplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RemoteIMAP_STARTTLSTest.test_logincapa did not pass"
print("RemoteIMAP_STARTTLSTest::test_logincapa: ok")
