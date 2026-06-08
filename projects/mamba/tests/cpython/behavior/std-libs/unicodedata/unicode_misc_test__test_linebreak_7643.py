# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_misc_test__test_linebreak_7643"
# subject = "cpython.test_unicodedata.UnicodeMiscTest.test_linebreak_7643"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeMiscTest::test_linebreak_7643
"""Auto-ported test: UnicodeMiscTest::test_linebreak_7643 (CPython 3.12 oracle)."""


import hashlib
from http.client import HTTPException
import sys
import unicodedata
import unittest
from test.support import open_urlresource, requires_resource, script_helper, cpython_only, check_disallow_instantiation


' Tests for the unicodedata module.\n\n    Written by Marc-Andre Lemburg (mal@lemburg.com).\n\n    (c) Copyright CNRI, All Rights Reserved. NO WARRANTY.\n\n'

class UnicodeDatabaseTest(unittest.TestCase):
    db = unicodedata


# --- test body ---
db = unicodedata
for i in range(65536):
    lines = (chr(i) + 'A').splitlines()
    if i in (10, 11, 12, 13, 133, 28, 29, 30, 8232, 8233):

        assert len(lines) == 2
    else:

        assert len(lines) == 1
print("UnicodeMiscTest::test_linebreak_7643: ok")
