# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_v_s_o_c_k_test__testVSOCKConstants"
# subject = "cpython.test_socket.BasicVSOCKTest.testVSOCKConstants"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicVSOCKTest.testVSOCKConstants", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicVSOCKTest.testVSOCKConstants did not pass"
print("BasicVSOCKTest::testVSOCKConstants: ok")
