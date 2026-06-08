# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "grp"
# dimension = "behavior"
# case = "group_database_test_case__test_noninteger_gid_ucf56d47"
# subject = "cpython.test_grp.GroupDatabaseTestCase.test_noninteger_gid"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_grp.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_grp
_suite = unittest.defaultTestLoader.loadTestsFromName("GroupDatabaseTestCase.test_noninteger_gid", test_grp)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GroupDatabaseTestCase.test_noninteger_gid did not pass"
print("GroupDatabaseTestCase::test_noninteger_gid: ok")
