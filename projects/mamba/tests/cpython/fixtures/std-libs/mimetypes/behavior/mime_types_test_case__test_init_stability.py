# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_init_stability"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_init_stability"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mimetypes.py::MimeTypesTestCase::test_init_stability
"""Auto-ported test: MimeTypesTestCase::test_init_stability (CPython 3.12 oracle)."""


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
mimetypes.init()
suffix_map = mimetypes.suffix_map
encodings_map = mimetypes.encodings_map
types_map = mimetypes.types_map
common_types = mimetypes.common_types
mimetypes.init()

assert suffix_map is not mimetypes.suffix_map

assert encodings_map is not mimetypes.encodings_map

assert types_map is not mimetypes.types_map

assert common_types is not mimetypes.common_types

assert suffix_map == mimetypes.suffix_map

assert encodings_map == mimetypes.encodings_map

assert types_map == mimetypes.types_map

assert common_types == mimetypes.common_types
print("MimeTypesTestCase::test_init_stability: ok")
