# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "behavior"
# case = "file_input_tests__test_file_opening_hook"
# subject = "cpython.test_fileinput.FileInputTests.test_file_opening_hook"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileinput.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileinput.py::FileInputTests::test_file_opening_hook
"""Auto-ported test: FileInputTests::test_file_opening_hook (CPython 3.12 oracle)."""


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
def writeTmp(content, *, mode='w'):
    fd, name = tempfile.mkstemp()
    pass
    encoding = None if 'b' in mode else 'utf-8'
    with open(fd, mode, encoding=encoding) as f:
        f.write(content)
    return name
try:
    fi = FileInput(inplace=1, openhook=lambda f, m: None)

    raise AssertionError('FileInput should raise if both inplace and openhook arguments are given')
except ValueError:
    pass
try:
    fi = FileInput(openhook=1)

    raise AssertionError('FileInput should check openhook for being callable')
except ValueError:
    pass

class CustomOpenHook:

    def __init__(self):
        self.invoked = False

    def __call__(self, *args, **kargs):
        self.invoked = True
        return open(*args, encoding='utf-8')
t = writeTmp('\n')
custom_open_hook = CustomOpenHook()
with FileInput([t], openhook=custom_open_hook) as fi:
    fi.readline()

assert custom_open_hook.invoked
print("FileInputTests::test_file_opening_hook: ok")
