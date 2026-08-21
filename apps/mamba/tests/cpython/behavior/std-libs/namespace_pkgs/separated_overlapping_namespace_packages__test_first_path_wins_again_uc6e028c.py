# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "namespace_pkgs"
# dimension = "behavior"
# case = "separated_overlapping_namespace_packages__test_first_path_wins_again_uc6e028c"
# subject = "cpython.test_namespace_pkgs.SeparatedOverlappingNamespacePackages.test_first_path_wins_again"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_namespace_pkgs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_namespace_pkgs
_suite = unittest.defaultTestLoader.loadTestsFromName("SeparatedOverlappingNamespacePackages.test_first_path_wins_again", test_namespace_pkgs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SeparatedOverlappingNamespacePackages.test_first_path_wins_again did not pass"
print("SeparatedOverlappingNamespacePackages::test_first_path_wins_again: ok")
