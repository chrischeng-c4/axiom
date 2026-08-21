# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_hyper_v_test__testCreateHyperVSocketAddrNotTupleOf2StrsFailure"
# subject = "cpython.test_socket.BasicHyperVTest.testCreateHyperVSocketAddrNotTupleOf2StrsFailure"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicHyperVTest.testCreateHyperVSocketAddrNotTupleOf2StrsFailure", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicHyperVTest.testCreateHyperVSocketAddrNotTupleOf2StrsFailure did not pass"
print("BasicHyperVTest::testCreateHyperVSocketAddrNotTupleOf2StrsFailure: ok")
