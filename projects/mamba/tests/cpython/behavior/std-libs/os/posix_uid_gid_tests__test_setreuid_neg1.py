# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "posix_uid_gid_tests__test_setreuid_neg1"
# subject = "cpython.test_os.PosixUidGidTests.test_setreuid_neg1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("PosixUidGidTests.test_setreuid_neg1", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PosixUidGidTests.test_setreuid_neg1 did not pass"
print("PosixUidGidTests::test_setreuid_neg1: ok")
