# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tuple"
# dimension = "behavior"
# case = "tuple_test__test_len_uc8c5972"
# subject = "cpython.test_tuple.TupleTest.test_len"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tuple
_suite = unittest.defaultTestLoader.loadTestsFromName("TupleTest.test_len", test_tuple)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TupleTest.test_len did not pass"
print("TupleTest::test_len: ok")
