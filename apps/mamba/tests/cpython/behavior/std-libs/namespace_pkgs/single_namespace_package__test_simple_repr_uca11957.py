# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "single_namespace_package__test_simple_repr_uca11957"
# subject = "cpython.test_namespace_pkgs.SingleNamespacePackage.test_simple_repr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("SingleNamespacePackage.test_simple_repr", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SingleNamespacePackage.test_simple_repr did not pass"
print("SingleNamespacePackage::test_simple_repr: ok")
