# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "embed"
# dimension = "behavior"
# case = "embedding_tests__test_specialized_static_code_gets_unspecialized_at_Py_FINALIZE"
# subject = "cpython.test_embed.EmbeddingTests.test_specialized_static_code_gets_unspecialized_at_Py_FINALIZE"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_embed.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_embed
_suite = unittest.defaultTestLoader.loadTestsFromName("EmbeddingTests.test_specialized_static_code_gets_unspecialized_at_Py_FINALIZE", test_embed)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EmbeddingTests.test_specialized_static_code_gets_unspecialized_at_Py_FINALIZE did not pass"
print("EmbeddingTests::test_specialized_static_code_gets_unspecialized_at_Py_FINALIZE: ok")
