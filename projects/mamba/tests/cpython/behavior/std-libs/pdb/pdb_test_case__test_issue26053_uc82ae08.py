# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "behavior"
# case = "pdb_test_case__test_issue26053_uc82ae08"
# subject = "cpython.test_pdb.PdbTestCase.test_issue26053"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pdb
_suite = unittest.defaultTestLoader.loadTestsFromName("PdbTestCase.test_issue26053", test_pdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PdbTestCase.test_issue26053 did not pass"
print("PdbTestCase::test_issue26053: ok")
