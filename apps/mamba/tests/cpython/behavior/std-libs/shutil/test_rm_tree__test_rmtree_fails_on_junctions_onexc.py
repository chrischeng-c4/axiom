# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_rm_tree__test_rmtree_fails_on_junctions_onexc"
# subject = "cpython.test_shutil.TestRmTree.test_rmtree_fails_on_junctions_onexc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestRmTree.test_rmtree_fails_on_junctions_onexc", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestRmTree.test_rmtree_fails_on_junctions_onexc did not pass"
print("TestRmTree::test_rmtree_fails_on_junctions_onexc: ok")
