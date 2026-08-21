# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "other_tests__test_struct_sizes"
# subject = "cpython.test_core.OtherTests.test_struct_sizes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
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

def create_zipfile_with_extra_data(filename, extra_data_name):
    with zipfile.ZipFile(TESTFN, mode='w') as zf:
        filename_encoded = filename.encode('utf-8')
        zip_info = zipfile.ZipInfo(filename)
        tag_for_unicode_path = b'up'
        version_of_unicode_path = b'\x01'
        import zlib
        filename_crc = struct.pack('<L', zlib.crc32(filename_encoded))
        extra_data = version_of_unicode_path + filename_crc + extra_data_name
        tsize = len(extra_data).to_bytes(2, 'little')
        zip_info.extra = tag_for_unicode_path + tsize + extra_data
        zf.writestr(zip_info, b'Hello World!')
'Check that ZIP internal structure sizes are calculated correctly.'
assert zipfile.sizeEndCentDir == 22
assert zipfile.sizeCentralDir == 46
assert zipfile.sizeEndCentDir64 == 56
assert zipfile.sizeEndCentDir64Locator == 20

print("OtherTests::test_struct_sizes: ok")
