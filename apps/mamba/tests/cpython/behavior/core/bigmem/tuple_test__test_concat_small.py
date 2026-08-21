# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bigmem"
# dimension = "behavior"
# case = "tuple_test__test_concat_small"
# subject = "cpython.test_bigmem.TupleTest.test_concat_small"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigmem.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigmem
_suite = unittest.defaultTestLoader.loadTestsFromName("TupleTest.test_concat_small", test_bigmem)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TupleTest.test_concat_small did not pass"
print("TupleTest::test_concat_small: ok")
