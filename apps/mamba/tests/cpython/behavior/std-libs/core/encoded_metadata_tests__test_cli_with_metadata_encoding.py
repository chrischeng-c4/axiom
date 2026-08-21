# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "encoded_metadata_tests__test_cli_with_metadata_encoding"
# subject = "cpython.test_core.EncodedMetadataTests.test_cli_with_metadata_encoding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("EncodedMetadataTests.test_cli_with_metadata_encoding", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EncodedMetadataTests.test_cli_with_metadata_encoding did not pass"
print("EncodedMetadataTests::test_cli_with_metadata_encoding: ok")
