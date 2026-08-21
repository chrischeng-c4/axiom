# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "embed"
# dimension = "behavior"
# case = "embedding_tests__test_static_types_inherited_slots"
# subject = "cpython.test_embed.EmbeddingTests.test_static_types_inherited_slots"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_embed.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_embed
_suite = unittest.defaultTestLoader.loadTestsFromName("EmbeddingTests.test_static_types_inherited_slots", test_embed)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EmbeddingTests.test_static_types_inherited_slots did not pass"
print("EmbeddingTests::test_static_types_inherited_slots: ok")
