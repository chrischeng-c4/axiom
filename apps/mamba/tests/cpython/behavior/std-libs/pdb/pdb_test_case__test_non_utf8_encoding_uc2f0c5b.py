# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "behavior"
# case = "pdb_test_case__test_non_utf8_encoding_uc2f0c5b"
# subject = "cpython.test_pdb.PdbTestCase.test_non_utf8_encoding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pdb
_suite = unittest.defaultTestLoader.loadTestsFromName("PdbTestCase.test_non_utf8_encoding", test_pdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PdbTestCase.test_non_utf8_encoding did not pass"
print("PdbTestCase::test_non_utf8_encoding: ok")
