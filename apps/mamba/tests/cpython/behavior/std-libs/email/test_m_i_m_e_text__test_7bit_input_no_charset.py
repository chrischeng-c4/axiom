# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_m_i_m_e_text__test_7bit_input_no_charset"
# subject = "cpython.test_email.TestMIMEText.test_7bit_input_no_charset"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMIMEText.test_7bit_input_no_charset", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMIMEText.test_7bit_input_no_charset did not pass"
print("TestMIMEText::test_7bit_input_no_charset: ok")
