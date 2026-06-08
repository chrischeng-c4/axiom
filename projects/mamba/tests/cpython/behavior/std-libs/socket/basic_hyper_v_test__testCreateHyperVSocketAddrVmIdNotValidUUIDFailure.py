# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_hyper_v_test__testCreateHyperVSocketAddrVmIdNotValidUUIDFailure"
# subject = "cpython.test_socket.BasicHyperVTest.testCreateHyperVSocketAddrVmIdNotValidUUIDFailure"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicHyperVTest.testCreateHyperVSocketAddrVmIdNotValidUUIDFailure", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicHyperVTest.testCreateHyperVSocketAddrVmIdNotValidUUIDFailure did not pass"
print("BasicHyperVTest::testCreateHyperVSocketAddrVmIdNotValidUUIDFailure: ok")
