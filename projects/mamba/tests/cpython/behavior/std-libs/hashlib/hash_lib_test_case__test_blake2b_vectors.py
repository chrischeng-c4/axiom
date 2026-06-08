# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "hash_lib_test_case__test_blake2b_vectors"
# subject = "cpython.test_hashlib.HashLibTestCase.test_blake2b_vectors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hashlib
_suite = unittest.defaultTestLoader.loadTestsFromName("HashLibTestCase.test_blake2b_vectors", test_hashlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HashLibTestCase.test_blake2b_vectors did not pass"
print("HashLibTestCase::test_blake2b_vectors: ok")
