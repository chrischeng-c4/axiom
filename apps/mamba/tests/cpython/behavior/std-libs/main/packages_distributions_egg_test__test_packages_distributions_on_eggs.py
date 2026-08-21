# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "packages_distributions_egg_test__test_packages_distributions_on_eggs"
# subject = "cpython.test_main.PackagesDistributionsEggTest.test_packages_distributions_on_eggs"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("PackagesDistributionsEggTest.test_packages_distributions_on_eggs", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PackagesDistributionsEggTest.test_packages_distributions_on_eggs did not pass"
print("PackagesDistributionsEggTest::test_packages_distributions_on_eggs: ok")
