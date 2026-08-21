# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "path"
# dimension = "behavior"
# case = "test_path__test_glob_subdir"
# subject = "cpython.test_path.TestPath.test_glob_subdir"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/_path/test_path.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile._path import test_path
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPath.test_glob_subdir", test_path)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPath.test_glob_subdir did not pass"
print("TestPath::test_glob_subdir: ok")
