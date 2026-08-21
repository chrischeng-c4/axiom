# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "separated_namespace_packages__test_imports_ucd21ac4"
# subject = "cpython.test_namespace_pkgs.SeparatedNamespacePackages.test_imports"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("SeparatedNamespacePackages.test_imports", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SeparatedNamespacePackages.test_imports did not pass"
print("SeparatedNamespacePackages::test_imports: ok")
