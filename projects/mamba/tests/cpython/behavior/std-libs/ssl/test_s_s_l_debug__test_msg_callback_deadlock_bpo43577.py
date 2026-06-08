# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "test_s_s_l_debug__test_msg_callback_deadlock_bpo43577"
# subject = "cpython.test_ssl.TestSSLDebug.test_msg_callback_deadlock_bpo43577"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSSLDebug.test_msg_callback_deadlock_bpo43577", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSSLDebug.test_msg_callback_deadlock_bpo43577 did not pass"
print("TestSSLDebug::test_msg_callback_deadlock_bpo43577: ok")
