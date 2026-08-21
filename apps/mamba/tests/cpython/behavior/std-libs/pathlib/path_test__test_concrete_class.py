# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "path_test__test_concrete_class"
# subject = "cpython.test_pathlib.PathTest.test_concrete_class"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pathlib
_suite = unittest.defaultTestLoader.loadTestsFromName("PathTest.test_concrete_class", test_pathlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PathTest.test_concrete_class did not pass"
print("PathTest::test_concrete_class: ok")
