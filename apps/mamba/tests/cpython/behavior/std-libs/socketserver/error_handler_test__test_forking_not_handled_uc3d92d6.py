# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "behavior"
# case = "error_handler_test__test_forking_not_handled_uc3d92d6"
# subject = "cpython.test_socketserver.ErrorHandlerTest.test_forking_not_handled"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socketserver.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socketserver
_suite = unittest.defaultTestLoader.loadTestsFromName("ErrorHandlerTest.test_forking_not_handled", test_socketserver)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ErrorHandlerTest.test_forking_not_handled did not pass"
print("ErrorHandlerTest::test_forking_not_handled: ok")
