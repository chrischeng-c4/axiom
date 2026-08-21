# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "write_test__test_abs_pathnames"
# subject = "cpython.test_tarfile.WriteTest.test_abs_pathnames"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("WriteTest.test_abs_pathnames", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WriteTest.test_abs_pathnames did not pass"
print("WriteTest::test_abs_pathnames: ok")
