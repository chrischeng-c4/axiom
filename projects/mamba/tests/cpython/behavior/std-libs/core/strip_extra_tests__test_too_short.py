# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "strip_extra_tests__test_too_short"
# subject = "cpython.test_core.StripExtraTests.test_too_short"
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
assert b'' == zipfile._strip_extra(b'', (ZIP64_EXTRA,))
assert b'z' == zipfile._strip_extra(b'z', (ZIP64_EXTRA,))
assert b'zz' == zipfile._strip_extra(b'zz', (ZIP64_EXTRA,))
assert b'zzz' == zipfile._strip_extra(b'zzz', (ZIP64_EXTRA,))

print("StripExtraTests::test_too_short: ok")
