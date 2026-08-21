# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_counter__test_copying_ucaf649a"
# subject = "cpython.test_collections.TestCounter.test_copying"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_collections
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCounter.test_copying", test_collections)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCounter.test_copying did not pass"
print("TestCounter::test_copying: ok")
