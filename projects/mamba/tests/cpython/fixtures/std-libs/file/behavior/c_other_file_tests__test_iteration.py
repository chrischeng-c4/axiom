# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_other_file_tests__test_iteration"
# subject = "cpython.test_file.COtherFileTests.testIteration"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::COtherFileTests::testIteration
"""Auto-ported test: COtherFileTests::testIteration (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = io.open

def _checkBufferSize(s):
    try:
        f = open(TESTFN, 'wb', s)
        f.write(str(s).encode('ascii'))
        f.close()
        f.close()
        f = open(TESTFN, 'rb', s)
        d = int(f.read().decode('ascii'))
        f.close()
        f.close()
    except OSError as msg:

        raise AssertionError('error setting buffer size %d: %s' % (s, str(msg)))

    assert d == s
dataoffset = 16384
filler = b'ham\n'
assert not dataoffset % len(filler), 'dataoffset must be multiple of len(filler)'
nchunks = dataoffset // len(filler)
testlines = [b'spam, spam and eggs\n', b'eggs, spam, ham and spam\n', b'saussages, spam, spam and eggs\n', b'spam, ham, spam and eggs\n', b'spam, spam, spam, spam, spam, ham, spam\n', b'wonderful spaaaaaam.\n']
methods = [('readline', ()), ('read', ()), ('readlines', ()), ('readinto', (array('b', b' ' * 100),))]
bag = open(TESTFN, 'wb')
bag.write(filler * nchunks)
bag.writelines(testlines)
bag.close()
for methodname, args in methods:
    f = open(TESTFN, 'rb')

    assert next(f) == filler
    meth = getattr(f, methodname)
    meth(*args)
    f.close()
f = open(TESTFN, 'rb')
for i in range(nchunks):
    next(f)
testline = testlines.pop(0)
try:
    line = f.readline()
except ValueError:

    raise AssertionError('readline() after next() with supposedly empty iteration-buffer failed anyway')
if line != testline:

    raise AssertionError('readline() after next() with empty buffer failed. Got %r, expected %r' % (line, testline))
testline = testlines.pop(0)
buf = array('b', b'\x00' * len(testline))
try:
    f.readinto(buf)
except ValueError:

    raise AssertionError('readinto() after next() with supposedly empty iteration-buffer failed anyway')
line = buf.tobytes()
if line != testline:

    raise AssertionError('readinto() after next() with empty buffer failed. Got %r, expected %r' % (line, testline))
testline = testlines.pop(0)
try:
    line = f.read(len(testline))
except ValueError:

    raise AssertionError('read() after next() with supposedly empty iteration-buffer failed anyway')
if line != testline:

    raise AssertionError('read() after next() with empty buffer failed. Got %r, expected %r' % (line, testline))
try:
    lines = f.readlines()
except ValueError:

    raise AssertionError('readlines() after next() with supposedly empty iteration-buffer failed anyway')
if lines != testlines:

    raise AssertionError('readlines() after next() with empty buffer failed. Got %r, expected %r' % (line, testline))
f.close()
f = open(TESTFN, 'rb')
try:
    for line in f:
        pass
    try:
        f.readline()
        f.readinto(buf)
        f.read()
        f.readlines()
    except ValueError:

        raise AssertionError('read* failed after next() consumed file')
finally:
    f.close()
print("COtherFileTests::testIteration: ok")
