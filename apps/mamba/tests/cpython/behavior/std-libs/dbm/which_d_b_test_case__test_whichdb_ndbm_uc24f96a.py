# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "behavior"
# case = "which_d_b_test_case__test_whichdb_ndbm_uc24f96a"
# subject = "cpython.test_dbm.WhichDBTestCase.test_whichdb_ndbm"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dbm
_suite = unittest.defaultTestLoader.loadTestsFromName("WhichDBTestCase.test_whichdb_ndbm", test_dbm)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WhichDBTestCase.test_whichdb_ndbm did not pass"
print("WhichDBTestCase::test_whichdb_ndbm: ok")
