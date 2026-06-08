# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_functions_test__test_east_asian_width_unassigned"
# subject = "cpython.test_unicodedata.UnicodeFunctionsTest.test_east_asian_width_unassigned"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeFunctionsTest::test_east_asian_width_unassigned
"""Auto-ported test: UnicodeFunctionsTest::test_east_asian_width_unassigned (CPython 3.12 oracle)."""


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
eaw = db.east_asian_width
for char in '\u0530\u0ecf\u10c6\u20fc\uaaca\U000107bd\U000115f2':

    assert eaw(char) == 'N'

    assert db.name(char, None) is None
for char in '\ufa6e\ufada\U0002a6e0\U0002fa20\U0003134b\U0003fffd':

    assert eaw(char) == 'W'

    assert db.name(char, None) is None
for char in '\ue000\uf800\U000f0000\U000fffee\U00100000\U0010fff0':

    assert eaw(char) == 'A'

    assert db.name(char, None) is None
print("UnicodeFunctionsTest::test_east_asian_width_unassigned: ok")
