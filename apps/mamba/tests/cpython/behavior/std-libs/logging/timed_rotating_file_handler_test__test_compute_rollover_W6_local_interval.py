# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "timed_rotating_file_handler_test__test_compute_rollover_W6_local_interval"
# subject = "cpython.test_logging.TimedRotatingFileHandlerTest.test_compute_rollover_W6_local_interval"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("TimedRotatingFileHandlerTest.test_compute_rollover_W6_local_interval", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TimedRotatingFileHandlerTest.test_compute_rollover_W6_local_interval did not pass"
print("TimedRotatingFileHandlerTest::test_compute_rollover_W6_local_interval: ok")
