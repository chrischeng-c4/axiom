# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_collection_a_b_cs__test_buffer_ucb1591f"
# subject = "cpython.test_collections.TestCollectionABCs.test_Buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_collections
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCollectionABCs.test_Buffer", test_collections)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCollectionABCs.test_Buffer did not pass"
print("TestCollectionABCs::test_Buffer: ok")
