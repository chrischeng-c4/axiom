# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "strip_extra_tests__test_with_data"
# subject = "cpython.test_core.StripExtraTests.test_with_data"
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
ZIP64_EXTRA = 1
s = struct.Struct('<HH')
a = s.pack(ZIP64_EXTRA, 1) + b'a'
b = s.pack(2, 2) + b'bb'
c = s.pack(3, 3) + b'ccc'
assert b'' == zipfile._strip_extra(a, (ZIP64_EXTRA,))
assert b == zipfile._strip_extra(b, (ZIP64_EXTRA,))
assert b + b'z' == zipfile._strip_extra(b + b'z', (ZIP64_EXTRA,))
assert b + c == zipfile._strip_extra(a + b + c, (ZIP64_EXTRA,))
assert b + c == zipfile._strip_extra(b + a + c, (ZIP64_EXTRA,))
assert b + c == zipfile._strip_extra(b + c + a, (ZIP64_EXTRA,))

print("StripExtraTests::test_with_data: ok")
