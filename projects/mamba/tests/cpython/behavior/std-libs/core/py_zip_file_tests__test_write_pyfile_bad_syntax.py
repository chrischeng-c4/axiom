# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "py_zip_file_tests__test_write_pyfile_bad_syntax"
# subject = "cpython.test_core.PyZipFileTests.test_write_pyfile_bad_syntax"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("PyZipFileTests.test_write_pyfile_bad_syntax", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyZipFileTests.test_write_pyfile_bad_syntax did not pass"
print("PyZipFileTests::test_write_pyfile_bad_syntax: ok")
