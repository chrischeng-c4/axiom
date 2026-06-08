# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_guess_all_types"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_guess_all_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mimetypes.py::MimeTypesTestCase::test_guess_all_types
"""Auto-ported test: MimeTypesTestCase::test_guess_all_types (CPython 3.12 oracle)."""


import io
import mimetypes
import os
import sys
import unittest.mock
from test import support
from test.support import os_helper
from platform import win32_edition


try:
    import _winapi
except ImportError:
    _winapi = None

def setUpModule():
    global knownfiles
    knownfiles = mimetypes.knownfiles
    mimetypes.knownfiles = []
    mimetypes.inited = False
    mimetypes._default_mime_types()

def tearDownModule():
    mimetypes.knownfiles = knownfiles


# --- test body ---
self_db = mimetypes.MimeTypes()
all = self_db.guess_all_extensions('text/plain', strict=True)

assert set(all) >= {'.bat', '.c', '.h', '.ksh', '.pl', '.txt'}

assert len(set(all)) == len(all)
all = self_db.guess_all_extensions('image/jpg', strict=False)

assert all == ['.jpg']
all = self_db.guess_all_extensions('image/jpg', strict=True)

assert all == []
self_db.add_type('test-type', '.strict-ext')
self_db.add_type('test-type', '.non-strict-ext', strict=False)
all = self_db.guess_all_extensions('test-type', strict=False)

assert all == ['.strict-ext', '.non-strict-ext']
all = self_db.guess_all_extensions('test-type')

assert all == ['.strict-ext']
all.append('.no-such-ext')
all = self_db.guess_all_extensions('test-type')

assert '.no-such-ext' not in all
print("MimeTypesTestCase::test_guess_all_types: ok")
