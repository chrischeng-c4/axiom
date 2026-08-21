# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "strip_extra_tests__test_multiples"
# subject = "cpython.test_core.StripExtraTests.test_multiples"
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
assert b'' == zipfile._strip_extra(a + a, (ZIP64_EXTRA,))
assert b'' == zipfile._strip_extra(a + a + a, (ZIP64_EXTRA,))
assert b'z' == zipfile._strip_extra(a + a + b'z', (ZIP64_EXTRA,))
assert b + b'z' == zipfile._strip_extra(a + a + b + b'z', (ZIP64_EXTRA,))
assert b == zipfile._strip_extra(a + a + b, (ZIP64_EXTRA,))
assert b == zipfile._strip_extra(a + b + a, (ZIP64_EXTRA,))
assert b == zipfile._strip_extra(b + a + a, (ZIP64_EXTRA,))

print("StripExtraTests::test_multiples: ok")
