# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bigmem"
# dimension = "behavior"
# case = "list_test__test_hash"
# subject = "cpython.test_bigmem.ListTest.test_hash"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigmem.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigmem
_suite = unittest.defaultTestLoader.loadTestsFromName("ListTest.test_hash", test_bigmem)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ListTest.test_hash did not pass"
print("ListTest::test_hash: ok")
