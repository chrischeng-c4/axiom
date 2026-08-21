# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "reload_tests__test_simple_package_uc1f6379"
# subject = "cpython.test_namespace_pkgs.ReloadTests.test_simple_package"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("ReloadTests.test_simple_package", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReloadTests.test_simple_package did not pass"
print("ReloadTests::test_simple_package: ok")
