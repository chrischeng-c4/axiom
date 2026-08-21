# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_misc_test__test_decimal_numeric_consistent"
# subject = "cpython.test_unicodedata.UnicodeMiscTest.test_decimal_numeric_consistent"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeMiscTest::test_decimal_numeric_consistent
"""Auto-ported test: UnicodeMiscTest::test_decimal_numeric_consistent (CPython 3.12 oracle)."""


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
count = 0
for i in range(65536):
    c = chr(i)
    dec = db.decimal(c, -1)
    if dec != -1:

        assert dec == db.numeric(c)
        count += 1

assert count >= 10
print("UnicodeMiscTest::test_decimal_numeric_consistent: ok")
