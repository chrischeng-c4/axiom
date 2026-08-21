# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "test_named_tuple__test_field_doc_reuse_ucd4cdd7"
# subject = "cpython.test_collections.TestNamedTuple.test_field_doc_reuse"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_collections.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_collections
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNamedTuple.test_field_doc_reuse", test_collections)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNamedTuple.test_field_doc_reuse did not pass"
print("TestNamedTuple::test_field_doc_reuse: ok")
