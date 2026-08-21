# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "u_d_p_l_i_t_e_timeout_test__testTimeoutZero"
# subject = "cpython.test_socket.UDPLITETimeoutTest.testTimeoutZero"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("UDPLITETimeoutTest.testTimeoutZero", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UDPLITETimeoutTest.testTimeoutZero did not pass"
print("UDPLITETimeoutTest::testTimeoutZero: ok")
