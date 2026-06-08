# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_xxsubinterpreters"
# dimension = "behavior"
# case = "create_tests__test_in_thread_uc44aa80"
# subject = "cpython.test__xxsubinterpreters.CreateTests.test_in_thread"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__xxsubinterpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__xxsubinterpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("CreateTests.test_in_thread", test__xxsubinterpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CreateTests.test_in_thread did not pass"
print("CreateTests::test_in_thread: ok")
