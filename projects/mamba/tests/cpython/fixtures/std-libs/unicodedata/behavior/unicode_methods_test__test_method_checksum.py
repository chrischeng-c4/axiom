# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_methods_test__test_method_checksum"
# subject = "cpython.test_unicodedata.UnicodeMethodsTest.test_method_checksum"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeMethodsTest::test_method_checksum
"""Auto-ported test: UnicodeMethodsTest::test_method_checksum (CPython 3.12 oracle)."""


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
expectedchecksum = 'e708c31c0d51f758adf475cb7201cf80917362be'
h = hashlib.sha1()
for i in range(sys.maxunicode + 1):
    char = chr(i)
    data = ['01'[char.isalnum()], '01'[char.isalpha()], '01'[char.isdecimal()], '01'[char.isdigit()], '01'[char.islower()], '01'[char.isnumeric()], '01'[char.isspace()], '01'[char.istitle()], '01'[char.isupper()], '01'[(char + 'abc').isalnum()], '01'[(char + 'abc').isalpha()], '01'[(char + '123').isdecimal()], '01'[(char + '123').isdigit()], '01'[(char + 'abc').islower()], '01'[(char + '123').isnumeric()], '01'[(char + ' \t').isspace()], '01'[(char + 'abc').istitle()], '01'[(char + 'ABC').isupper()], char.lower(), char.upper(), char.title(), (char + 'abc').lower(), (char + 'ABC').upper(), (char + 'abc').title(), (char + 'ABC').title()]
    h.update(''.join(data).encode('utf-8', 'surrogatepass'))
result = h.hexdigest()

assert result == expectedchecksum
print("UnicodeMethodsTest::test_method_checksum: ok")
