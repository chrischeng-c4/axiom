# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "behavior"
# case = "test_timeit__test_timeit_callable_stmt_uc8a3a5a"
# subject = "cpython.test_timeit.TestTimeit.test_timeit_callable_stmt"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_timeit
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTimeit.test_timeit_callable_stmt", test_timeit)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTimeit.test_timeit_callable_stmt did not pass"
print("TestTimeit::test_timeit_callable_stmt: ok")
