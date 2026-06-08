# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sax"
# dimension = "behavior"
# case = "expat_reader_test__test_expat_nsattrs_wattr"
# subject = "cpython.test_sax.ExpatReaderTest.test_expat_nsattrs_wattr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sax
_suite = unittest.defaultTestLoader.loadTestsFromName("ExpatReaderTest.test_expat_nsattrs_wattr", test_sax)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExpatReaderTest.test_expat_nsattrs_wattr did not pass"
print("ExpatReaderTest::test_expat_nsattrs_wattr: ok")
