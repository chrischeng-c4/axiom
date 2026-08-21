# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "bz2_detect_read_test__test_detect_stream_bz2"
# subject = "cpython.test_tarfile.Bz2DetectReadTest.test_detect_stream_bz2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("Bz2DetectReadTest.test_detect_stream_bz2", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Bz2DetectReadTest.test_detect_stream_bz2 did not pass"
print("Bz2DetectReadTest::test_detect_stream_bz2: ok")
