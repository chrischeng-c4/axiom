# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "stream_reader_test__test_copy"
# subject = "cpython.test_codecs.StreamReaderTest.test_copy"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("StreamReaderTest.test_copy", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StreamReaderTest.test_copy did not pass"
print("StreamReaderTest::test_copy: ok")
