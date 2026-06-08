# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "behavior"
# case = "fault_handler_tests__test_stderr_None"
# subject = "cpython.test_faulthandler.FaultHandlerTests.test_stderr_None"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_faulthandler.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_faulthandler
_suite = unittest.defaultTestLoader.loadTestsFromName("FaultHandlerTests.test_stderr_None", test_faulthandler)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FaultHandlerTests.test_stderr_None did not pass"
print("FaultHandlerTests::test_stderr_None: ok")
