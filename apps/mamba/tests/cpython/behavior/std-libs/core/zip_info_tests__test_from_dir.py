# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "zip_info_tests__test_from_dir"
# subject = "cpython.test_core.ZipInfoTests.test_from_dir"
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
dirpath = os.path.dirname(os.path.abspath(__file__))
zi = zipfile.ZipInfo.from_file(dirpath, 'stdlib_tests')
assert zi.filename == 'stdlib_tests/'
assert zi.is_dir()
assert zi.compress_type == zipfile.ZIP_STORED
assert zi.file_size == 0

print("ZipInfoTests::test_from_dir: ok")
