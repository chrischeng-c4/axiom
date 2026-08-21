# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "zip_info_tests__test_from_file_fileno"
# subject = "cpython.test_core.ZipInfoTests.test_from_file_fileno"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import _pyio
import array
import contextlib
import importlib.util
import io
import itertools
import os
import posixpath
import struct
import subprocess
import sys
import time
import zipfile
from tempfile import TemporaryFile
from random import randint, random, randbytes
with open(__file__, 'rb') as f:
    zi = zipfile.ZipInfo.from_file(f.fileno(), 'test')
    assert posixpath.basename(zi.filename) == 'test'
    assert not zi.is_dir()
    assert zi.file_size == os.path.getsize(__file__)

print("ZipInfoTests::test_from_file_fileno: ok")
