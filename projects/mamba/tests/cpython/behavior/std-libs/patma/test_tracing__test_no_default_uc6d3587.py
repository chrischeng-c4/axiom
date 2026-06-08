# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "patma"
# dimension = "behavior"
# case = "test_tracing__test_no_default_uc6d3587"
# subject = "cpython.test_patma.TestTracing.test_no_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_patma
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTracing.test_no_default", test_patma)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTracing.test_no_default did not pass"
print("TestTracing::test_no_default: ok")
