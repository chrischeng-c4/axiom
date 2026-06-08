# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_functions_test__test_pr29"
# subject = "cpython.test_unicodedata.UnicodeFunctionsTest.test_pr29"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeFunctionsTest::test_pr29
"""Auto-ported test: UnicodeFunctionsTest::test_pr29 (CPython 3.12 oracle)."""


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
expectedchecksum = '26ff0d31c14194b4606a5b3a81ac36df3a14e331'
composed = ('େ̀ା', 'ᄀ̀ᅡ', 'Li̍t-sṳ́', 'मार्क ज़' + 'ुकेरबर्ग', 'किर्गिज़' + 'स्तान')
for text in composed:

    assert db.normalize('NFC', text) == text
print("UnicodeFunctionsTest::test_pr29: ok")
