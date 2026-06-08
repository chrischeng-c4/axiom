# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "helpers"
# dimension = "behavior"
# case = "test_bless_my_loader__test_gh86298_no_loader_and_no_spec_ucf719f2"
# subject = "cpython.test_helpers.TestBlessMyLoader.test_gh86298_no_loader_and_no_spec"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/import_/test_helpers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.import_ import test_helpers
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBlessMyLoader.test_gh86298_no_loader_and_no_spec", test_helpers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBlessMyLoader.test_gh86298_no_loader_and_no_spec did not pass"
print("TestBlessMyLoader::test_gh86298_no_loader_and_no_spec: ok")
