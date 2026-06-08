# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "behavior"
# case = "buffer_sizes_tests__test_buffer_sizes"
# subject = "cpython.test_fileinput.BufferSizesTests.test_buffer_sizes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileinput.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileinput.py::BufferSizesTests::test_buffer_sizes
"""Auto-ported test: BufferSizesTests::test_buffer_sizes (CPython 3.12 oracle)."""


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
t1 = writeTmp(''.join(('Line %s of file 1\n' % (i + 1) for i in range(15))))
t2 = writeTmp(''.join(('Line %s of file 2\n' % (i + 1) for i in range(10))))
t3 = writeTmp(''.join(('Line %s of file 3\n' % (i + 1) for i in range(5))))
t4 = writeTmp(''.join(('Line %s of file 4\n' % (i + 1) for i in range(1))))
pat = re.compile('LINE (\\d+) OF FILE (\\d+)')
if verbose:
    print('1. Simple iteration')
fi = FileInput(files=(t1, t2, t3, t4), encoding='utf-8')
lines = list(fi)
fi.close()

assert len(lines) == 31

assert lines[4] == 'Line 5 of file 1\n'

assert lines[30] == 'Line 1 of file 4\n'

assert fi.lineno() == 31

assert fi.filename() == t4
if verbose:
    print('2. Status variables')
fi = FileInput(files=(t1, t2, t3, t4), encoding='utf-8')
s = 'x'
while s and s != 'Line 6 of file 2\n':
    s = fi.readline()

assert fi.filename() == t2

assert fi.lineno() == 21

assert fi.filelineno() == 6

assert not fi.isfirstline()

assert not fi.isstdin()
if verbose:
    print('3. Nextfile')
fi.nextfile()

assert fi.readline() == 'Line 1 of file 3\n'

assert fi.lineno() == 22
fi.close()
if verbose:
    print('4. Stdin')
fi = FileInput(files=(t1, t2, t3, t4, '-'), encoding='utf-8')
savestdin = sys.stdin
try:
    sys.stdin = StringIO('Line 1 of stdin\nLine 2 of stdin\n')
    lines = list(fi)

    assert len(lines) == 33

    assert lines[32] == 'Line 2 of stdin\n'

    assert fi.filename() == '<stdin>'
    fi.nextfile()
finally:
    sys.stdin = savestdin
if verbose:
    print('5. Boundary conditions')
fi = FileInput(files=(t1, t2, t3, t4), encoding='utf-8')

assert fi.lineno() == 0

assert fi.filename() == None
fi.nextfile()

assert fi.lineno() == 0

assert fi.filename() == None
if verbose:
    print('6. Inplace')
savestdout = sys.stdout
try:
    fi = FileInput(files=(t1, t2, t3, t4), inplace=1, encoding='utf-8')
    for line in fi:
        line = line[:-1].upper()
        print(line)
    fi.close()
finally:
    sys.stdout = savestdout
fi = FileInput(files=(t1, t2, t3, t4), encoding='utf-8')
for line in fi:

    assert line[-1] == '\n'
    m = pat.match(line[:-1])

    assert m != None

    assert int(m.group(1)) == fi.filelineno()
fi.close()
print("BufferSizesTests::test_buffer_sizes: ok")
