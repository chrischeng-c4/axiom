# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "ustar_unicode_test__test_unicode_longname1"
# subject = "cpython.test_tarfile.UstarUnicodeTest.test_unicode_longname1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("UstarUnicodeTest.test_unicode_longname1", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UstarUnicodeTest.test_unicode_longname1 did not pass"
print("UstarUnicodeTest::test_unicode_longname1: ok")
