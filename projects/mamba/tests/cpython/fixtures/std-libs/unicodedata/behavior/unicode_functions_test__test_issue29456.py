# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_functions_test__test_issue29456"
# subject = "cpython.test_unicodedata.UnicodeFunctionsTest.test_issue29456"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeFunctionsTest::test_issue29456
"""Auto-ported test: UnicodeFunctionsTest::test_issue29456 (CPython 3.12 oracle)."""


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
u1176_str_a = 'ᄀᅶᆨ'
u1176_str_b = 'ᄀᅶᆨ'
u11a7_str_a = '기ᆧ'
u11a7_str_b = '기ᆧ'
u11c3_str_a = '기ᇃ'
u11c3_str_b = '기ᇃ'

assert db.normalize('NFC', u1176_str_a) == u1176_str_b

assert db.normalize('NFC', u11a7_str_a) == u11a7_str_b

assert db.normalize('NFC', u11c3_str_a) == u11c3_str_b
print("UnicodeFunctionsTest::test_issue29456: ok")
