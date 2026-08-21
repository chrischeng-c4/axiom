# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "posix_tester__test_putenv"
# subject = "cpython.test_posix.PosixTester.test_putenv"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_posix
_suite = unittest.defaultTestLoader.loadTestsFromName("PosixTester.test_putenv", test_posix)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PosixTester.test_putenv did not pass"
print("PosixTester::test_putenv: ok")
