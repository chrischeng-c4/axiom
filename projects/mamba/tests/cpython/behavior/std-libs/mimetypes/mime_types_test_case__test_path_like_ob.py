# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_path_like_ob"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_path_like_ob"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mimetypes.py::MimeTypesTestCase::test_path_like_ob
"""Auto-ported test: MimeTypesTestCase::test_path_like_ob (CPython 3.12 oracle)."""


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
filename = 'LICENSE.txt'
filepath = os_helper.FakePath(filename)
filepath_with_abs_dir = os_helper.FakePath('/dir/' + filename)
filepath_relative = os_helper.FakePath('../dir/' + filename)
path_dir = os_helper.FakePath('./')
expected = self_db.guess_type(filename)

assert self_db.guess_type(filepath) == expected

assert self_db.guess_type(filepath_with_abs_dir) == expected

assert self_db.guess_type(filepath_relative) == expected

assert self_db.guess_type(path_dir) == (None, None)
print("MimeTypesTestCase::test_path_like_ob: ok")
