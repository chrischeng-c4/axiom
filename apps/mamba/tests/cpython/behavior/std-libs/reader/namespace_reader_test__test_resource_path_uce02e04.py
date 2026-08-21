# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reader"
# dimension = "behavior"
# case = "namespace_reader_test__test_resource_path_uce02e04"
# subject = "cpython.test_reader.NamespaceReaderTest.test_resource_path"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_reader.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_reader
_suite = unittest.defaultTestLoader.loadTestsFromName("NamespaceReaderTest.test_resource_path", test_reader)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NamespaceReaderTest.test_resource_path did not pass"
print("NamespaceReaderTest::test_resource_path: ok")
