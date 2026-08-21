# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "module_level_misc_test__test_subclass_logger_cache"
# subject = "cpython.test_logging.ModuleLevelMiscTest.test_subclass_logger_cache"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("ModuleLevelMiscTest.test_subclass_logger_cache", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ModuleLevelMiscTest.test_subclass_logger_cache did not pass"
print("ModuleLevelMiscTest::test_subclass_logger_cache: ok")
