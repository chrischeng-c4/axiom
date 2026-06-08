# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "behavior"
# case = "test_fileinput_lineno__test_state_is_none"
# subject = "cpython.test_fileinput.Test_fileinput_lineno.test_state_is_None"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileinput.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileinput.py::Test_fileinput_lineno::test_state_is_None
"""Auto-ported test: Test_fileinput_lineno::test_state_is_None (CPython 3.12 oracle)."""


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

class BaseFileInputGlobalMethodsTest(unittest.TestCase):
    """Base class for unit tests for the global function of
       the fileinput module."""

    def setUp(self):
        self._orig_state = fileinput._state
        self._orig_FileInput = fileinput.FileInput
        fileinput.FileInput = MockFileInput

    def tearDown(self):
        fileinput.FileInput = self._orig_FileInput
        fileinput._state = self._orig_state

    def assertExactlyOneInvocation(self, mock_file_input, method_name):
        actual_count = mock_file_input.invocation_counts[method_name]
        self.assertEqual(actual_count, 1, method_name)
        actual_total_count = len(mock_file_input.invocation_counts)
        self.assertEqual(actual_total_count, 1)

class InvocationRecorder:

    def __init__(self):
        self.invocation_count = 0

    def __call__(self, *args, **kwargs):
        self.invocation_count += 1
        self.last_invocation = (args, kwargs)
        return io.BytesIO(b'some bytes')


# --- test body ---
self__orig_state = fileinput._state
self__orig_FileInput = fileinput.FileInput
fileinput.FileInput = MockFileInput
'Tests fileinput.lineno() when fileinput._state is None.\n           Ensure that it raises RuntimeError with a meaningful error message\n           and does not modify fileinput._state'
fileinput._state = None
try:
    fileinput.lineno()
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert ('no active input()',) == cm.exception.args

assert fileinput._state is None
print("Test_fileinput_lineno::test_state_is_None: ok")
