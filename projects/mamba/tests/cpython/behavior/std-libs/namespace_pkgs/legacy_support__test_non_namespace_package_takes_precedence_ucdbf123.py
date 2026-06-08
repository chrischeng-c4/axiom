# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "legacy_support__test_non_namespace_package_takes_precedence_ucdbf123"
# subject = "cpython.test_namespace_pkgs.LegacySupport.test_non_namespace_package_takes_precedence"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("LegacySupport.test_non_namespace_package_takes_precedence", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LegacySupport.test_non_namespace_package_takes_precedence did not pass"
print("LegacySupport::test_non_namespace_package_takes_precedence: ok")
