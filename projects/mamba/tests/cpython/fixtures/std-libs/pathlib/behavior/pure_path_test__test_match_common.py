# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_match_common"
# subject = "cpython.test_pathlib.PurePathTest.test_match_common"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_match_common
"""Auto-ported test: PurePathTest::test_match_common (CPython 3.12 oracle)."""


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
cls = pathlib.PurePath

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

try:
    P('a').match('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a').match('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert P('b.py').match('b.py')

assert P('a/b.py').match('b.py')

assert P('/a/b.py').match('b.py')

assert not P('a.py').match('b.py')

assert not P('b/py').match('b.py')

assert not P('/a.py').match('b.py')

assert not P('b.py/c').match('b.py')

assert P('b.py').match('*.py')

assert P('a/b.py').match('*.py')

assert P('/a/b.py').match('*.py')

assert not P('b.pyc').match('*.py')

assert not P('b./py').match('*.py')

assert not P('b.py/c').match('*.py')

assert P('ab/c.py').match('a*/*.py')

assert P('/d/ab/c.py').match('a*/*.py')

assert not P('a.py').match('a*/*.py')

assert not P('/dab/c.py').match('a*/*.py')

assert not P('ab/c.py/d').match('a*/*.py')

assert P('/b.py').match('/*.py')

assert not P('b.py').match('/*.py')

assert not P('a/b.py').match('/*.py')

assert not P('/a/b.py').match('/*.py')

assert P('/a/b.py').match('/a/*.py')

assert not P('/ab.py').match('/a/*.py')

assert not P('/a/b/c.py').match('/a/*.py')

assert not P('/a/b/c.py').match('/**/*.py')

assert P('/a/b/c.py').match('/a/**/*.py')

assert not P('A.py').match('a.PY', case_sensitive=True)

assert P('A.py').match('a.PY', case_sensitive=False)

assert not P('c:/a/B.Py').match('C:/A/*.pY', case_sensitive=True)

assert P('/a/b/c.py').match('/A/*/*.Py', case_sensitive=False)

assert not P().match('*')

assert P().match('**')

assert not P().match('**/*')
print("PurePathTest::test_match_common: ok")
