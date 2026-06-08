# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_misc_test__test_ucd_510"
# subject = "cpython.test_unicodedata.UnicodeMiscTest.test_ucd_510"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeMiscTest::test_ucd_510
"""Auto-ported test: UnicodeMiscTest::test_ucd_510 (CPython 3.12 oracle)."""


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
import unicodedata

assert unicodedata.mirrored('༺')

assert not unicodedata.ucd_3_2_0.mirrored('༺')

assert 'a'.upper() == 'A'

assert 'ᵹ'.upper() == 'Ᵹ'

assert '.'.upper() == '.'
print("UnicodeMiscTest::test_ucd_510: ok")
