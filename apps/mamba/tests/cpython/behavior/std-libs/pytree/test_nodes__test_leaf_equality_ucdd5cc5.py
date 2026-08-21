# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pytree"
# dimension = "behavior"
# case = "test_nodes__test_leaf_equality_ucdd5cc5"
# subject = "cpython.test_pytree.TestNodes.test_leaf_equality"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_pytree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_pytree
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNodes.test_leaf_equality", test_pytree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNodes.test_leaf_equality did not pass"
print("TestNodes::test_leaf_equality: ok")
