# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "helpers"
# dimension = "behavior"
# case = "test_bless_my_loader__test_gh86298_no_loader_with_spec_loader_okay_ucdba73d"
# subject = "cpython.test_helpers.TestBlessMyLoader.test_gh86298_no_loader_with_spec_loader_okay"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/import_/test_helpers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.import_ import test_helpers
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBlessMyLoader.test_gh86298_no_loader_with_spec_loader_okay", test_helpers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBlessMyLoader.test_gh86298_no_loader_with_spec_loader_okay did not pass"
print("TestBlessMyLoader::test_gh86298_no_loader_with_spec_loader_okay: ok")
