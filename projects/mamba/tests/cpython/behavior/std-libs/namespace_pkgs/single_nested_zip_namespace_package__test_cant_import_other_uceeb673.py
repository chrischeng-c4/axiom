# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "single_nested_zip_namespace_package__test_cant_import_other_uceeb673"
# subject = "cpython.test_namespace_pkgs.SingleNestedZipNamespacePackage.test_cant_import_other"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("SingleNestedZipNamespacePackage.test_cant_import_other", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SingleNestedZipNamespacePackage.test_cant_import_other did not pass"
print("SingleNestedZipNamespacePackage::test_cant_import_other: ok")
