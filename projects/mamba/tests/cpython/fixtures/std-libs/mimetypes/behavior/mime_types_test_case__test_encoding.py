# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_encoding"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_encoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mimetypes.py::MimeTypesTestCase::test_encoding
"""Auto-ported test: MimeTypesTestCase::test_encoding (CPython 3.12 oracle)."""


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
filename = support.findfile('mime.types')
mimes = mimetypes.MimeTypes([filename])
exts = mimes.guess_all_extensions('application/vnd.geocube+xml', strict=True)

assert exts == ['.g3', '.g³']
print("MimeTypesTestCase::test_encoding: ok")
