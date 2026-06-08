# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "other_tests__test_create_empty_zipinfo_default_attributes"
# subject = "cpython.test_core.OtherTests.test_create_empty_zipinfo_default_attributes"
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
'Ensure all required attributes are set.'
zi = zipfile.ZipInfo()
assert zi.orig_filename == 'NoName'
assert zi.filename == 'NoName'
assert zi.date_time == (1980, 1, 1, 0, 0, 0)
assert zi.compress_type == zipfile.ZIP_STORED
assert zi.comment == b''
assert zi.extra == b''
assert zi.create_system in (0, 3)
assert zi.create_version == zipfile.DEFAULT_VERSION
assert zi.extract_version == zipfile.DEFAULT_VERSION
assert zi.reserved == 0
assert zi.flag_bits == 0
assert zi.volume == 0
assert zi.internal_attr == 0
assert zi.external_attr == 0
assert zi.file_size == 0
assert zi.compress_size == 0

print("OtherTests::test_create_empty_zipinfo_default_attributes: ok")
