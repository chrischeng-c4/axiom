# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "behavior"
# case = "zip_app_test__test_main_and_main_py"
# subject = "cpython.test_zipapp.ZipAppTest.test_main_and_main_py"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipapp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipapp.py::ZipAppTest::test_main_and_main_py
"""Auto-ported test: ZipAppTest::test_main_and_main_py (CPython 3.12 oracle)."""


import io
import pathlib
import stat
import sys
import tempfile
import unittest
import zipapp
import zipfile
from test.support import requires_zlib
from test.support import os_helper
from unittest.mock import patch


'Test harness for the zipapp module.'


# --- test body ---
tmpdir = tempfile.TemporaryDirectory()
pass
self_tmpdir = pathlib.Path(tmpdir.name)
source = self_tmpdir / 'source'
source.mkdir()
(source / '__main__.py').touch()
target = self_tmpdir / 'source.pyz'
try:
    zipapp.create_archive(str(source), str(target), main='pkg.mod:fn')
    raise AssertionError('expected zipapp.ZipAppError')
except zipapp.ZipAppError:
    pass
print("ZipAppTest::test_main_and_main_py: ok")
