# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_bluetooth_test__testCreateRfcommSocket"
# subject = "cpython.test_socket.BasicBluetoothTest.testCreateRfcommSocket"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicBluetoothTest.testCreateRfcommSocket", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicBluetoothTest.testCreateRfcommSocket did not pass"
print("BasicBluetoothTest::testCreateRfcommSocket: ok")
