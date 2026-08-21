# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bigaddrspace"
# dimension = "behavior"
# case = "str_test__test_optimized_concat_ucd650fc"
# subject = "cpython.test_bigaddrspace.StrTest.test_optimized_concat"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigaddrspace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigaddrspace
_suite = unittest.defaultTestLoader.loadTestsFromName("StrTest.test_optimized_concat", test_bigaddrspace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StrTest.test_optimized_concat did not pass"
print("StrTest::test_optimized_concat: ok")
