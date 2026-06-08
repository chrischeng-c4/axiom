# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "path"
# dimension = "behavior"
# case = "path_disk_tests__test_natural_path"
# subject = "cpython.test_path.PathDiskTests.test_natural_path"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_path.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_path
_suite = unittest.defaultTestLoader.loadTestsFromName("PathDiskTests.test_natural_path", test_path)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PathDiskTests.test_natural_path did not pass"
print("PathDiskTests::test_natural_path: ok")
