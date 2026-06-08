# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "behavior"
# case = "test_timeit__test_main_setup_uc35503e"
# subject = "cpython.test_timeit.TestTimeit.test_main_setup"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_timeit
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTimeit.test_main_setup", test_timeit)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTimeit.test_main_setup did not pass"
print("TestTimeit::test_main_setup: ok")
