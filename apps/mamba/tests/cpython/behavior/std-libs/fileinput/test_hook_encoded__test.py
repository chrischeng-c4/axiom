# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "behavior"
# case = "test_hook_encoded__test"
# subject = "cpython.test_fileinput.Test_hook_encoded.test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileinput.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileinput.py::Test_hook_encoded::test
"""Auto-ported test: Test_hook_encoded::test (CPython 3.12 oracle)."""


import io
import os
import sys
import re
import fileinput
import collections
import builtins
import tempfile
import unittest
from io import BytesIO, StringIO
from fileinput import FileInput, hook_encoded
from test.support import verbose
from test.support.os_helper import TESTFN, FakePath
from test.support.os_helper import unlink as safe_unlink
from test.support import os_helper
from test import support
from unittest import mock


'\nTests for fileinput module.\nNick Mathewson\n'

try:
    import bz2
except ImportError:
    bz2 = None

try:
    import gzip
except ImportError:
    gzip = None

class BaseTests:

    def writeTmp(self, content, *, mode='w'):
        fd, name = tempfile.mkstemp()
        self.addCleanup(os_helper.unlink, name)
        encoding = None if 'b' in mode else 'utf-8'
        with open(fd, mode, encoding=encoding) as f:
            f.write(content)
        return name

class LineReader:

    def __init__(self):
        self._linesread = []

    @property
    def linesread(self):
        try:
            return self._linesread[:]
        finally:
            self._linesread = []

    def openhook(self, filename, mode):
        self.it = iter(filename.splitlines(True))
        return self

    def readline(self, size=None):
        line = next(self.it, '')
        self._linesread.append(line)
        return line

    def readlines(self, hint=-1):
        lines = []
        size = 0
        while True:
            line = self.readline()
            if not line:
                return lines
            lines.append(line)
            size += len(line)
            if size >= hint:
                return lines

    def close(self):
        pass

class UnconditionallyRaise:

    def __init__(self, exception_type):
        self.exception_type = exception_type
        self.invoked = False

    def __call__(self, *args, **kwargs):
        self.invoked = True
        raise self.exception_type()

class MockFileInput:
    """A class that mocks out fileinput.FileInput for use during unit tests"""

    def __init__(self, files=None, inplace=False, backup='', *, mode='r', openhook=None, encoding=None, errors=None):
        self.files = files
        self.inplace = inplace
        self.backup = backup
        self.mode = mode
        self.openhook = openhook
        self.encoding = encoding
        self.errors = errors
        self._file = None
        self.invocation_counts = collections.defaultdict(lambda: 0)
        self.return_values = {}

    def close(self):
        self.invocation_counts['close'] += 1

    def nextfile(self):
        self.invocation_counts['nextfile'] += 1
        return self.return_values['nextfile']

    def filename(self):
        self.invocation_counts['filename'] += 1
        return self.return_values['filename']

    def lineno(self):
        self.invocation_counts['lineno'] += 1
        return self.return_values['lineno']

    def filelineno(self):
        self.invocation_counts['filelineno'] += 1
        return self.return_values['filelineno']

    def fileno(self):
        self.invocation_counts['fileno'] += 1
        return self.return_values['fileno']

    def isfirstline(self):
        self.invocation_counts['isfirstline'] += 1
        return self.return_values['isfirstline']

    def isstdin(self):
        self.invocation_counts['isstdin'] += 1
        return self.return_values['isstdin']

class InvocationRecorder:

    def __init__(self):
        self.invocation_count = 0

    def __call__(self, *args, **kwargs):
        self.invocation_count += 1
        self.last_invocation = (args, kwargs)
        return io.BytesIO(b'some bytes')


# --- test body ---
encoding = object()
errors = object()
result = fileinput.hook_encoded(encoding, errors=errors)
fake_open = InvocationRecorder()
original_open = builtins.open
builtins.open = fake_open
try:
    filename = object()
    mode = object()
    open_result = result(filename, mode)
finally:
    builtins.open = original_open

assert fake_open.invocation_count == 1
args, kwargs = fake_open.last_invocation

assert args[0] is filename

assert args[1] is mode

assert kwargs.pop('encoding') is encoding

assert kwargs.pop('errors') is errors

assert not kwargs
print("Test_hook_encoded::test: ok")
