# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_keywords_args_api"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_keywords_args_api"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mimetypes.py::MimeTypesTestCase::test_keywords_args_api
"""Auto-ported test: MimeTypesTestCase::test_keywords_args_api (CPython 3.12 oracle)."""


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

assert self_db.guess_type(url='foo.html', strict=True) == ('text/html', None)

assert self_db.guess_all_extensions(type='image/jpg', strict=True) == []

assert self_db.guess_extension(type='image/jpg', strict=False) == '.jpg'
print("MimeTypesTestCase::test_keywords_args_api: ok")
