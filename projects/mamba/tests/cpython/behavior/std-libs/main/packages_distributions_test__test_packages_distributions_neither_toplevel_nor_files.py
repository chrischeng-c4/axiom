# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "packages_distributions_test__test_packages_distributions_neither_toplevel_nor_files"
# subject = "cpython.test_main.PackagesDistributionsTest.test_packages_distributions_neither_toplevel_nor_files"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("PackagesDistributionsTest.test_packages_distributions_neither_toplevel_nor_files", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PackagesDistributionsTest.test_packages_distributions_neither_toplevel_nor_files did not pass"
print("PackagesDistributionsTest::test_packages_distributions_neither_toplevel_nor_files: ok")
