# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_functions_test__test_category"
# subject = "cpython.test_unicodedata.UnicodeFunctionsTest.test_category"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeFunctionsTest::test_category
"""Auto-ported test: UnicodeFunctionsTest::test_category (CPython 3.12 oracle)."""


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

assert db.category('\ufffe') == 'Cn'

assert db.category('a') == 'Ll'

assert db.category('A') == 'Lu'

assert db.category('𠀀') == 'Lo'

assert db.category('𐄪') == 'No'

try:
    db.category()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    db.category('xx')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("UnicodeFunctionsTest::test_category: ok")
