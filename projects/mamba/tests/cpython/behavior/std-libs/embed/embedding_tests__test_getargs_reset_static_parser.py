# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "embed"
# dimension = "behavior"
# case = "embedding_tests__test_getargs_reset_static_parser"
# subject = "cpython.test_embed.EmbeddingTests.test_getargs_reset_static_parser"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_embed.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_embed
_suite = unittest.defaultTestLoader.loadTestsFromName("EmbeddingTests.test_getargs_reset_static_parser", test_embed)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EmbeddingTests.test_getargs_reset_static_parser did not pass"
print("EmbeddingTests::test_getargs_reset_static_parser: ok")
