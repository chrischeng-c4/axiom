# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "embed"
# dimension = "behavior"
# case = "misc_tests__test_unicode_id_init"
# subject = "cpython.test_embed.MiscTests.test_unicode_id_init"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_embed.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_embed
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTests.test_unicode_id_init", test_embed)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTests.test_unicode_id_init did not pass"
print("MiscTests::test_unicode_id_init: ok")
