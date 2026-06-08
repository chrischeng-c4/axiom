# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "normalization_test__test_edge_cases"
# subject = "cpython.test_unicodedata.NormalizationTest.test_edge_cases"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicodedata.py::NormalizationTest::test_edge_cases
"""Auto-ported test: NormalizationTest::test_edge_cases (CPython 3.12 oracle)."""


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

try:
    unicodedata.normalize()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    unicodedata.normalize('unknown', 'xx')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert unicodedata.normalize('NFKC', '') == ''
print("NormalizationTest::test_edge_cases: ok")
