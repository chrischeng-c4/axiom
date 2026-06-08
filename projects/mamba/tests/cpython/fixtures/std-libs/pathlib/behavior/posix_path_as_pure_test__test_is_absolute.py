# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_is_absolute"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_is_absolute"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_is_absolute
"""Auto-ported test: PosixPathAsPureTest::test_is_absolute (CPython 3.12 oracle)."""


import contextlib
import collections.abc
import io
import os
import sys
import errno
import pathlib
import pickle
import socket
import stat
import tempfile
import unittest
from unittest import mock
from test.support import import_helper
from test.support import set_recursion_limit
from test.support import is_emscripten, is_wasi
from test.support import os_helper
from test.support.os_helper import TESTFN, FakePath


try:
    import grp, pwd
except ImportError:
    grp = pwd = None

class _BasePurePathSubclass(object):

    def __init__(self, *pathsegments, session_id):
        super().__init__(*pathsegments)
        self.session_id = session_id

    def with_segments(self, *pathsegments):
        return type(self)(*pathsegments, session_id=self.session_id)

BASE = os.path.realpath(TESTFN)

join = lambda *x: os.path.join(BASE, *x)

rel_join = lambda *x: os.path.join(TESTFN, *x)

only_nt = unittest.skipIf(os.name != 'nt', 'test requires a Windows-compatible system')

only_posix = unittest.skipIf(os.name == 'nt', 'test requires a POSIX-compatible system')


# --- test body ---
equivalences = {'a/b': [('a', 'b'), ('a/', 'b'), ('a', 'b/'), ('a/', 'b/'), ('a/b/',), ('a//b',), ('a//b//',), ('', 'a', 'b'), ('a', '', 'b'), ('a', 'b', '')], '/b/c/d': [('a', '/b/c', 'd'), ('/a', '/b/c', 'd'), ('/', 'b', '', 'c/d'), ('/', '', 'b/c/d'), ('', '/b/c/d')]}
cls = pathlib.PurePosixPath
cls = pathlib.PosixPath

def _check_drive_root_parts(arg, *expected):
    sep = self_flavour.sep
    actual = _get_drive_root_parts([x.replace('/', sep) for x in arg])

    assert actual == expected
    if (altsep := self_flavour.altsep):
        actual = _get_drive_root_parts([x.replace('/', altsep) for x in arg])

        assert actual == expected

def _check_str(expected, args):
    p = cls(*args)

    assert str(p) == expected.replace('/', self_sep)

def _check_str_subclass(*args):

    class StrSubclass(str):
        pass
    P = cls
    p = P(*(StrSubclass(x) for x in args))

    assert p == P(*args)
    for part in p.parts:

        assert type(part) is str

def _get_drive_root_parts(parts):
    path = cls(*parts)
    return (path.drive, path.root, path.parts)
p = cls('a')
self_flavour = p._flavour
self_sep = self_flavour.sep
self_altsep = self_flavour.altsep
P = cls

assert not P().is_absolute()

assert not P('a').is_absolute()

assert not P('a/b/').is_absolute()

assert P('/').is_absolute()

assert P('/a').is_absolute()

assert P('/a/b/').is_absolute()

assert P('//a').is_absolute()

assert P('//a/b').is_absolute()
print("PosixPathAsPureTest::test_is_absolute: ok")
