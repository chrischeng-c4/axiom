# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "builtin_levels_test__test_flat"
# subject = "cpython.test_logging.BuiltinLevelsTest.test_flat"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("BuiltinLevelsTest.test_flat", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BuiltinLevelsTest.test_flat did not pass"
print("BuiltinLevelsTest::test_flat: ok")
