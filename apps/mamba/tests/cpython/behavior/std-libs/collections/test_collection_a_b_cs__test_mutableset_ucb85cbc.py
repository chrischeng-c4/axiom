# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_collection_a_b_cs__test_mutableset_ucb85cbc"
# subject = "cpython.test_collections.TestCollectionABCs.test_MutableSet"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_collections
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCollectionABCs.test_MutableSet", test_collections)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCollectionABCs.test_MutableSet did not pass"
print("TestCollectionABCs::test_MutableSet: ok")
