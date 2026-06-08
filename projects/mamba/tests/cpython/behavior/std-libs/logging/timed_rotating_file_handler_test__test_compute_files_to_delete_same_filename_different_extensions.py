# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "timed_rotating_file_handler_test__test_compute_files_to_delete_same_filename_different_extensions"
# subject = "cpython.test_logging.TimedRotatingFileHandlerTest.test_compute_files_to_delete_same_filename_different_extensions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("TimedRotatingFileHandlerTest.test_compute_files_to_delete_same_filename_different_extensions", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TimedRotatingFileHandlerTest.test_compute_files_to_delete_same_filename_different_extensions did not pass"
print("TimedRotatingFileHandlerTest::test_compute_files_to_delete_same_filename_different_extensions: ok")
