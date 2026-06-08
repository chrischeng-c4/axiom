# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "module_and_namespace_package_in_same_dir__test_module_before_namespace_package_uc688edc"
# subject = "cpython.test_namespace_pkgs.ModuleAndNamespacePackageInSameDir.test_module_before_namespace_package"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("ModuleAndNamespacePackageInSameDir.test_module_before_namespace_package", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ModuleAndNamespacePackageInSameDir.test_module_before_namespace_package did not pass"
print("ModuleAndNamespacePackageInSameDir::test_module_before_namespace_package: ok")
