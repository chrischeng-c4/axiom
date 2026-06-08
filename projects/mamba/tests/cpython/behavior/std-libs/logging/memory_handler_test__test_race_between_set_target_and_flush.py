# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "memory_handler_test__test_race_between_set_target_and_flush"
# subject = "cpython.test_logging.MemoryHandlerTest.test_race_between_set_target_and_flush"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("MemoryHandlerTest.test_race_between_set_target_and_flush", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MemoryHandlerTest.test_race_between_set_target_and_flush did not pass"
print("MemoryHandlerTest::test_race_between_set_target_and_flush: ok")
