# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "hash_lib_test_case__test_hexdigest"
# subject = "cpython.test_hashlib.HashLibTestCase.test_hexdigest"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hashlib
_suite = unittest.defaultTestLoader.loadTestsFromName("HashLibTestCase.test_hexdigest", test_hashlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HashLibTestCase.test_hexdigest did not pass"
print("HashLibTestCase::test_hexdigest: ok")
