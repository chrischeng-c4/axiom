# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "log_record_test__test_taskName_with_asyncio_imported"
# subject = "cpython.test_logging.LogRecordTest.test_taskName_with_asyncio_imported"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("LogRecordTest.test_taskName_with_asyncio_imported", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LogRecordTest.test_taskName_with_asyncio_imported did not pass"
print("LogRecordTest::test_taskName_with_asyncio_imported: ok")
