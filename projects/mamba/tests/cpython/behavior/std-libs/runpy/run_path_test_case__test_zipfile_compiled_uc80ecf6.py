# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runpy"
# dimension = "behavior"
# case = "run_path_test_case__test_zipfile_compiled_uc80ecf6"
# subject = "cpython.test_runpy.RunPathTestCase.test_zipfile_compiled"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_runpy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_runpy
_suite = unittest.defaultTestLoader.loadTestsFromName("RunPathTestCase.test_zipfile_compiled", test_runpy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RunPathTestCase.test_zipfile_compiled did not pass"
print("RunPathTestCase::test_zipfile_compiled: ok")
