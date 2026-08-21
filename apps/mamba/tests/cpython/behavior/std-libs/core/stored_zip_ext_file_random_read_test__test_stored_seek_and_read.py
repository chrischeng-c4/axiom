# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "stored_zip_ext_file_random_read_test__test_stored_seek_and_read"
# subject = "cpython.test_core.StoredZipExtFileRandomReadTest.test_stored_seek_and_read"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("StoredZipExtFileRandomReadTest.test_stored_seek_and_read", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StoredZipExtFileRandomReadTest.test_stored_seek_and_read did not pass"
print("StoredZipExtFileRandomReadTest::test_stored_seek_and_read: ok")
