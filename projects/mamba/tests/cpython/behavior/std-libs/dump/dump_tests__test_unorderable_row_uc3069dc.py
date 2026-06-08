# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dump"
# dimension = "behavior"
# case = "dump_tests__test_unorderable_row_uc3069dc"
# subject = "cpython.test_dump.DumpTests.test_unorderable_row"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dump.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dump
_suite = unittest.defaultTestLoader.loadTestsFromName("DumpTests.test_unorderable_row", test_dump)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DumpTests.test_unorderable_row did not pass"
print("DumpTests::test_unorderable_row: ok")
