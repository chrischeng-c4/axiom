# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_conflict_resolve__test_conflict_resolve_long_opts"
# subject = "cpython.test_optparse.TestConflictResolve.test_conflict_resolve_long_opts"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestConflictResolve.test_conflict_resolve_long_opts", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestConflictResolve.test_conflict_resolve_long_opts did not pass"
print("TestConflictResolve::test_conflict_resolve_long_opts: ok")
