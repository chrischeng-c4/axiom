# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "packages_distributions_prebuilt_test__test_packages_distributions_example"
# subject = "cpython.test_main.PackagesDistributionsPrebuiltTest.test_packages_distributions_example"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("PackagesDistributionsPrebuiltTest.test_packages_distributions_example", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PackagesDistributionsPrebuiltTest.test_packages_distributions_example did not pass"
print("PackagesDistributionsPrebuiltTest::test_packages_distributions_example: ok")
