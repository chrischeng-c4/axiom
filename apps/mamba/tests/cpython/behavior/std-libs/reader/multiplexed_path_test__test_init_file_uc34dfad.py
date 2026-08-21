# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reader"
# dimension = "behavior"
# case = "multiplexed_path_test__test_init_file_uc34dfad"
# subject = "cpython.test_reader.MultiplexedPathTest.test_init_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_reader.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_reader
_suite = unittest.defaultTestLoader.loadTestsFromName("MultiplexedPathTest.test_init_file", test_reader)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MultiplexedPathTest.test_init_file did not pass"
print("MultiplexedPathTest::test_init_file: ok")
