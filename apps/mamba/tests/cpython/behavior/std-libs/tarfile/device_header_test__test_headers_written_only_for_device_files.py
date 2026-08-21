# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "device_header_test__test_headers_written_only_for_device_files"
# subject = "cpython.test_tarfile.DeviceHeaderTest.test_headers_written_only_for_device_files"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("DeviceHeaderTest.test_headers_written_only_for_device_files", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DeviceHeaderTest.test_headers_written_only_for_device_files did not pass"
print("DeviceHeaderTest::test_headers_written_only_for_device_files: ok")
