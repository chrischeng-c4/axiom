# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "behavior"
# case = "zip_app_cmdline_test__test_cmdline_create"
# subject = "cpython.test_zipapp.ZipAppCmdlineTest.test_cmdline_create"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipapp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipapp.py::ZipAppCmdlineTest::test_cmdline_create
"""Auto-ported test: ZipAppCmdlineTest::test_cmdline_create (CPython 3.12 oracle)."""


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
def make_archive():
    source = self_tmpdir / 'source'
    source.mkdir()
    (source / '__main__.py').touch()
    target = self_tmpdir / 'source.pyz'
    zipapp.create_archive(source, target)
    return target
tmpdir = tempfile.TemporaryDirectory()
pass
self_tmpdir = pathlib.Path(tmpdir.name)
source = self_tmpdir / 'source'
source.mkdir()
(source / '__main__.py').touch()
args = [str(source)]
zipapp.main(args)
target = source.with_suffix('.pyz')

assert target.is_file()
print("ZipAppCmdlineTest::test_cmdline_create: ok")
