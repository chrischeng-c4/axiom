# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bigmem"
# dimension = "behavior"
# case = "tuple_test__test_repeat_large_2"
# subject = "cpython.test_bigmem.TupleTest.test_repeat_large_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigmem.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigmem
_suite = unittest.defaultTestLoader.loadTestsFromName("TupleTest.test_repeat_large_2", test_bigmem)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TupleTest.test_repeat_large_2 did not pass"
print("TupleTest::test_repeat_large_2: ok")
