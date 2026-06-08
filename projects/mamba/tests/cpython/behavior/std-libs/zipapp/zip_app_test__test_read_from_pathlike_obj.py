# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "behavior"
# case = "zip_app_test__test_read_from_pathlike_obj"
# subject = "cpython.test_zipapp.ZipAppTest.test_read_from_pathlike_obj"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipapp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipapp.py::ZipAppTest::test_read_from_pathlike_obj
"""Auto-ported test: ZipAppTest::test_read_from_pathlike_obj (CPython 3.12 oracle)."""


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
source = os_helper.FakePath(str(source))
target1 = os_helper.FakePath(str(self_tmpdir / 'target1.pyz'))
target2 = os_helper.FakePath(str(self_tmpdir / 'target2.pyz'))
zipapp.create_archive(source, target1, interpreter='python')
zipapp.create_archive(target1, target2, interpreter='python2.7')

assert zipapp.get_interpreter(target2) == 'python2.7'
print("ZipAppTest::test_read_from_pathlike_obj: ok")
