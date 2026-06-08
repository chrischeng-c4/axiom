# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "rotating_file_handler_test__test_namer_rotator_inheritance"
# subject = "cpython.test_logging.RotatingFileHandlerTest.test_namer_rotator_inheritance"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("RotatingFileHandlerTest.test_namer_rotator_inheritance", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RotatingFileHandlerTest.test_namer_rotator_inheritance did not pass"
print("RotatingFileHandlerTest::test_namer_rotator_inheritance: ok")
