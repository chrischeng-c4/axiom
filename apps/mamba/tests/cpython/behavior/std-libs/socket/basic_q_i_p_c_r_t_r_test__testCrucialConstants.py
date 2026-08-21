# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_q_i_p_c_r_t_r_test__testCrucialConstants"
# subject = "cpython.test_socket.BasicQIPCRTRTest.testCrucialConstants"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicQIPCRTRTest.testCrucialConstants", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicQIPCRTRTest.testCrucialConstants did not pass"
print("BasicQIPCRTRTest::testCrucialConstants: ok")
