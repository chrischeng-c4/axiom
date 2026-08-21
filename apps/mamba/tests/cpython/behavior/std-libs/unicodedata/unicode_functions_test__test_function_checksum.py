# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "unicode_functions_test__test_function_checksum"
# subject = "cpython.test_unicodedata.UnicodeFunctionsTest.test_function_checksum"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::UnicodeFunctionsTest::test_function_checksum
"""Auto-ported test: UnicodeFunctionsTest::test_function_checksum (CPython 3.12 oracle)."""


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
data = []
h = hashlib.sha1()
for i in range(sys.maxunicode + 1):
    char = chr(i)
    data = [format(db.digit(char, -1), '.12g'), format(db.numeric(char, -1), '.12g'), format(db.decimal(char, -1), '.12g'), db.category(char), db.bidirectional(char), db.decomposition(char), str(db.mirrored(char)), str(db.combining(char)), unicodedata.east_asian_width(char), db.name(char, '')]
    h.update(''.join(data).encode('ascii'))
result = h.hexdigest()

assert result == expectedchecksum
print("UnicodeFunctionsTest::test_function_checksum: ok")
