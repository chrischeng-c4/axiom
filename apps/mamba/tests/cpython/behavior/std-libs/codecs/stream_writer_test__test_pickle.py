# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "stream_writer_test__test_pickle"
# subject = "cpython.test_codecs.StreamWriterTest.test_pickle"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("StreamWriterTest.test_pickle", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StreamWriterTest.test_pickle did not pass"
print("StreamWriterTest::test_pickle: ok")
