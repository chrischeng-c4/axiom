# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "behavior"
# case = "zip_app_test__test_create_archive_filter_exclude_dir"
# subject = "cpython.test_zipapp.ZipAppTest.test_create_archive_filter_exclude_dir"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipapp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipapp.py::ZipAppTest::test_create_archive_filter_exclude_dir
"""Auto-ported test: ZipAppTest::test_create_archive_filter_exclude_dir (CPython 3.12 oracle)."""


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

def skip_dummy_dir(path):
    return path.parts[0] != 'dummy'
source = self_tmpdir / 'source'
source.mkdir()
(source / '__main__.py').touch()
(source / 'test.py').touch()
(source / 'dummy').mkdir()
(source / 'dummy' / 'test2.py').touch()
target = self_tmpdir / 'source.pyz'
zipapp.create_archive(source, target, filter=skip_dummy_dir)
with zipfile.ZipFile(target, 'r') as z:

    assert len(z.namelist()) == 2

    assert '__main__.py' in z.namelist()

    assert 'test.py' in z.namelist()
print("ZipAppTest::test_create_archive_filter_exclude_dir: ok")
