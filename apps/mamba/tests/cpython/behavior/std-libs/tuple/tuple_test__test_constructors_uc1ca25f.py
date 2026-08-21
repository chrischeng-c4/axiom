# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tuple"
# dimension = "behavior"
# case = "tuple_test__test_constructors_uc1ca25f"
# subject = "cpython.test_tuple.TupleTest.test_constructors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tuple.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tuple
_suite = unittest.defaultTestLoader.loadTestsFromName("TupleTest.test_constructors", test_tuple)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TupleTest.test_constructors did not pass"
print("TupleTest::test_constructors: ok")
