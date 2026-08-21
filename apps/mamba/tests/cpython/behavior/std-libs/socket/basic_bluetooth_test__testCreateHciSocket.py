# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_bluetooth_test__testCreateHciSocket"
# subject = "cpython.test_socket.BasicBluetoothTest.testCreateHciSocket"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicBluetoothTest.testCreateHciSocket", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicBluetoothTest.testCreateHciSocket did not pass"
print("BasicBluetoothTest::testCreateHciSocket: ok")
