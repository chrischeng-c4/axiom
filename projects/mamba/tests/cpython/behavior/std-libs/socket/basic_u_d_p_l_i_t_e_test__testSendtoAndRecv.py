# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_u_d_p_l_i_t_e_test__testSendtoAndRecv"
# subject = "cpython.test_socket.BasicUDPLITETest.testSendtoAndRecv"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicUDPLITETest.testSendtoAndRecv", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicUDPLITETest.testSendtoAndRecv did not pass"
print("BasicUDPLITETest::testSendtoAndRecv: ok")
