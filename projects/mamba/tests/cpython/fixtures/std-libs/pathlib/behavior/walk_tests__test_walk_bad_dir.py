# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "walk_tests__test_walk_bad_dir"
# subject = "cpython.test_pathlib.WalkTests.test_walk_bad_dir"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pathlib.py::WalkTests::test_walk_bad_dir
"""Auto-ported test: WalkTests::test_walk_bad_dir (CPython 3.12 oracle)."""


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
pass
self_walk_path = pathlib.Path(os_helper.TESTFN, 'TEST1')
self_sub1_path = self_walk_path / 'SUB1'
self_sub11_path = self_sub1_path / 'SUB11'
self_sub2_path = self_walk_path / 'SUB2'
sub21_path = self_sub2_path / 'SUB21'
tmp1_path = self_walk_path / 'tmp1'
tmp2_path = self_sub1_path / 'tmp2'
tmp3_path = self_sub2_path / 'tmp3'
tmp5_path = sub21_path / 'tmp3'
self_link_path = self_sub2_path / 'link'
t2_path = pathlib.Path(os_helper.TESTFN, 'TEST2')
tmp4_path = pathlib.Path(os_helper.TESTFN, 'TEST2', 'tmp4')
broken_link_path = self_sub2_path / 'broken_link'
broken_link2_path = self_sub2_path / 'broken_link2'
broken_link3_path = self_sub2_path / 'broken_link3'
os.makedirs(self_sub11_path)
os.makedirs(self_sub2_path)
os.makedirs(sub21_path)
os.makedirs(t2_path)
for path in (tmp1_path, tmp2_path, tmp3_path, tmp4_path, tmp5_path):
    with open(path, 'x', encoding='utf-8') as f:
        f.write(f"I'm {path} and proud of it.  Blame test_pathlib.\n")
if os_helper.can_symlink():
    os.symlink(os.path.abspath(t2_path), self_link_path)
    os.symlink('broken', broken_link_path, True)
    os.symlink(pathlib.Path('tmp3', 'broken'), broken_link2_path, True)
    os.symlink(pathlib.Path('SUB21', 'tmp5'), broken_link3_path, True)
    self_sub2_tree = (self_sub2_path, ['SUB21'], ['broken_link', 'broken_link2', 'broken_link3', 'link', 'tmp3'])
else:
    self_sub2_tree = (self_sub2_path, ['SUB21'], ['tmp3'])
if not is_emscripten:
    os.chmod(sub21_path, 0)
try:
    os.listdir(sub21_path)
except PermissionError:
    pass
else:
    os.chmod(sub21_path, stat.S_IRWXU)
    os.unlink(tmp5_path)
    os.rmdir(sub21_path)
    del self_sub2_tree[1][:1]
errors = []
walk_it = self_walk_path.walk(on_error=errors.append)
root, dirs, files = next(walk_it)

assert errors == []
dir1 = 'SUB1'
path1 = root / dir1
path1new = (root / dir1).with_suffix('.new')
path1.rename(path1new)
try:
    roots = [r for r, _, _ in walk_it]

    assert errors

    assert path1 not in roots

    assert path1new not in roots
    for dir2 in dirs:
        if dir2 != dir1:

            assert root / dir2 in roots
finally:
    path1new.rename(path1)
print("WalkTests::test_walk_bad_dir: ok")
