# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "grp"
# dimension = "behavior"
# case = "group_database_test_case__test_errors_uc447ec5"
# subject = "cpython.test_grp.GroupDatabaseTestCase.test_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_grp.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_grp
_suite = unittest.defaultTestLoader.loadTestsFromName("GroupDatabaseTestCase.test_errors", test_grp)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GroupDatabaseTestCase.test_errors did not pass"
print("GroupDatabaseTestCase::test_errors: ok")
