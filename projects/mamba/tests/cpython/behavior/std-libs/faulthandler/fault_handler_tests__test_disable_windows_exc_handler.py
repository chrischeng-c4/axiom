# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "behavior"
# case = "fault_handler_tests__test_disable_windows_exc_handler"
# subject = "cpython.test_faulthandler.FaultHandlerTests.test_disable_windows_exc_handler"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_faulthandler.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_faulthandler
_suite = unittest.defaultTestLoader.loadTestsFromName("FaultHandlerTests.test_disable_windows_exc_handler", test_faulthandler)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FaultHandlerTests.test_disable_windows_exc_handler did not pass"
print("FaultHandlerTests::test_disable_windows_exc_handler: ok")
