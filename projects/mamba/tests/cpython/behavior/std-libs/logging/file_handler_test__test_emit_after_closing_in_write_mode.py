# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "file_handler_test__test_emit_after_closing_in_write_mode"
# subject = "cpython.test_logging.FileHandlerTest.test_emit_after_closing_in_write_mode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("FileHandlerTest.test_emit_after_closing_in_write_mode", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FileHandlerTest.test_emit_after_closing_in_write_mode did not pass"
print("FileHandlerTest::test_emit_after_closing_in_write_mode: ok")
