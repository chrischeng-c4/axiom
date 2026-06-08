# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "bz2_create_test__test_create_with_compresslevel"
# subject = "cpython.test_tarfile.Bz2CreateTest.test_create_with_compresslevel"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("Bz2CreateTest.test_create_with_compresslevel", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Bz2CreateTest.test_create_with_compresslevel did not pass"
print("Bz2CreateTest::test_create_with_compresslevel: ok")
