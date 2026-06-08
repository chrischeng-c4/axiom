# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "files"
# dimension = "behavior"
# case = "implicit_context_files_tests__test_implicit_files_uc7d1575"
# subject = "cpython.test_files.ImplicitContextFilesTests.test_implicit_files"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_files.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_files
_suite = unittest.defaultTestLoader.loadTestsFromName("ImplicitContextFilesTests.test_implicit_files", test_files)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImplicitContextFilesTests.test_implicit_files did not pass"
print("ImplicitContextFilesTests::test_implicit_files: ok")
