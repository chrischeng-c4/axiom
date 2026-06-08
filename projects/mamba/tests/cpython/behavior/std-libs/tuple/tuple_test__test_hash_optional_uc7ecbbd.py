# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tuple"
# dimension = "behavior"
# case = "tuple_test__test_hash_optional_uc7ecbbd"
# subject = "cpython.test_tuple.TupleTest.test_hash_optional"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tuple
_suite = unittest.defaultTestLoader.loadTestsFromName("TupleTest.test_hash_optional", test_tuple)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TupleTest.test_hash_optional did not pass"
print("TupleTest::test_hash_optional: ok")
