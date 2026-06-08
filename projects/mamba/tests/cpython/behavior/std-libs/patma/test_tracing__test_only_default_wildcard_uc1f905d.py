# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "patma"
# dimension = "behavior"
# case = "test_tracing__test_only_default_wildcard_uc1f905d"
# subject = "cpython.test_patma.TestTracing.test_only_default_wildcard"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_patma
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTracing.test_only_default_wildcard", test_patma)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTracing.test_only_default_wildcard did not pass"
print("TestTracing::test_only_default_wildcard: ok")
