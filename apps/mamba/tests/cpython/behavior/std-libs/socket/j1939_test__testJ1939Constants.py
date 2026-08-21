# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "j1939_test__testJ1939Constants"
# subject = "cpython.test_socket.J1939Test.testJ1939Constants"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("J1939Test.testJ1939Constants", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython J1939Test.testJ1939Constants did not pass"
print("J1939Test::testJ1939Constants: ok")
