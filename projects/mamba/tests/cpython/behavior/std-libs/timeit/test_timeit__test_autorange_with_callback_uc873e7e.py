# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "behavior"
# case = "test_timeit__test_autorange_with_callback_uc873e7e"
# subject = "cpython.test_timeit.TestTimeit.test_autorange_with_callback"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_timeit
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTimeit.test_autorange_with_callback", test_timeit)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTimeit.test_autorange_with_callback did not pass"
print("TestTimeit::test_autorange_with_callback: ok")
