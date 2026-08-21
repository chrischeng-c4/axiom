# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "test_tokenizer_adheres_to_pep0263__test_latin1_coding_cookie_and_utf8_bom_uc0cfb66"
# subject = "cpython.test_tokenize.TestTokenizerAdheresToPep0263.test_latin1_coding_cookie_and_utf8_bom"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tokenize
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTokenizerAdheresToPep0263.test_latin1_coding_cookie_and_utf8_bom", test_tokenize)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTokenizerAdheresToPep0263.test_latin1_coding_cookie_and_utf8_bom did not pass"
print("TestTokenizerAdheresToPep0263::test_latin1_coding_cookie_and_utf8_bom: ok")
