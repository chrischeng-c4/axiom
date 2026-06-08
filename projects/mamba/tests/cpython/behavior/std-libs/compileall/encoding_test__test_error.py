# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "encoding_test__test_error"
# subject = "cpython.test_compileall.EncodingTest.test_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import compileall
import contextlib
import filecmp
import importlib.util
import io
import os
import py_compile
import shutil
import struct
import sys
import tempfile
import time
self_directory = tempfile.mkdtemp()
self_source_path = os.path.join(self_directory, '_test.py')
with open(self_source_path, 'w', encoding='utf-8') as file:
    file.write('b"€"')
buffer = io.TextIOWrapper(io.BytesIO(), encoding='ascii')
with contextlib.redirect_stdout(buffer):
    compiled = compileall.compile_dir(self_directory)
assert not compiled
buffer.seek(0)
res = buffer.read()
assert 'SyntaxError: bytes can only contain ASCII literal characters' in res
assert 'UnicodeEncodeError' not in res

print("EncodingTest::test_error: ok")
