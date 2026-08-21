# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "posix_groups_tester__test_setgroups"
# subject = "cpython.test_posix.PosixGroupsTester.test_setgroups"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_posix
_suite = unittest.defaultTestLoader.loadTestsFromName("PosixGroupsTester.test_setgroups", test_posix)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PosixGroupsTester.test_setgroups did not pass"
print("PosixGroupsTester::test_setgroups: ok")
