# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_copy_tree__test_copytree_arg_types_of_ignore"
# subject = "cpython.test_shutil.TestCopyTree.test_copytree_arg_types_of_ignore"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCopyTree.test_copytree_arg_types_of_ignore", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCopyTree.test_copytree_arg_types_of_ignore did not pass"
print("TestCopyTree::test_copytree_arg_types_of_ignore: ok")
