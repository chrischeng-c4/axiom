# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "behavior"
# case = "zip_app_test__test_content_of_copied_archive"
# subject = "cpython.test_zipapp.ZipAppTest.test_content_of_copied_archive"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipapp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipapp.py::ZipAppTest::test_content_of_copied_archive
"""Auto-ported test: ZipAppTest::test_content_of_copied_archive (CPython 3.12 oracle)."""


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
target = io.BytesIO()
zipapp.create_archive(str(source), target, interpreter='python')
new_target = io.BytesIO()
target.seek(0)
zipapp.create_archive(target, new_target, interpreter=None)
new_target.seek(0)
with zipfile.ZipFile(new_target, 'r') as z:

    assert set(z.namelist()) == {'__main__.py'}
print("ZipAppTest::test_content_of_copied_archive: ok")
