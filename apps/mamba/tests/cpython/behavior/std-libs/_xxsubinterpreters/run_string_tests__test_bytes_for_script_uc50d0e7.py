# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_xxsubinterpreters"
# dimension = "behavior"
# case = "run_string_tests__test_bytes_for_script_uc50d0e7"
# subject = "cpython.test__xxsubinterpreters.RunStringTests.test_bytes_for_script"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__xxsubinterpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__xxsubinterpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("RunStringTests.test_bytes_for_script", test__xxsubinterpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RunStringTests.test_bytes_for_script did not pass"
print("RunStringTests::test_bytes_for_script: ok")
