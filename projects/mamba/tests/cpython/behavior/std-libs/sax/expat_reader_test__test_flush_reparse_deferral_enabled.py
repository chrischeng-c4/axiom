# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sax"
# dimension = "behavior"
# case = "expat_reader_test__test_flush_reparse_deferral_enabled"
# subject = "cpython.test_sax.ExpatReaderTest.test_flush_reparse_deferral_enabled"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sax
_suite = unittest.defaultTestLoader.loadTestsFromName("ExpatReaderTest.test_flush_reparse_deferral_enabled", test_sax)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExpatReaderTest.test_flush_reparse_deferral_enabled did not pass"
print("ExpatReaderTest::test_flush_reparse_deferral_enabled: ok")
