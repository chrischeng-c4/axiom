# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "test_send_recv__test_send_recv_different_interpreters_uc8c0525"
# subject = "cpython.test_interpreters.TestSendRecv.test_send_recv_different_interpreters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_interpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSendRecv.test_send_recv_different_interpreters", test_interpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSendRecv.test_send_recv_different_interpreters did not pass"
print("TestSendRecv::test_send_recv_different_interpreters: ok")
