# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "hash_lib_test_case__test_readonly_types"
# subject = "cpython.test_hashlib.HashLibTestCase.test_readonly_types"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hashlib
_suite = unittest.defaultTestLoader.loadTestsFromName("HashLibTestCase.test_readonly_types", test_hashlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HashLibTestCase.test_readonly_types did not pass"
print("HashLibTestCase::test_readonly_types: ok")
