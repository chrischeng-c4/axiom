# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "behavior"
# case = "fault_handler_tests__test_register_threads"
# subject = "cpython.test_faulthandler.FaultHandlerTests.test_register_threads"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_faulthandler.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_faulthandler
_suite = unittest.defaultTestLoader.loadTestsFromName("FaultHandlerTests.test_register_threads", test_faulthandler)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FaultHandlerTests.test_register_threads did not pass"
print("FaultHandlerTests::test_register_threads: ok")
