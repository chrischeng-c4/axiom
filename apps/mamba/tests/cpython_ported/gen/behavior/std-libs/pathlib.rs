use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/pathlib/anchor_absolute_vs_relative.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_anchor_absolute_vs_relative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "anchor_absolute_vs_relative"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('/a/b').anchor == '/' (absolute root) while Path('a/b').anchor == '' (relative has no anchor)"""
import pathlib

Path = pathlib.Path

assert Path("/a/b").anchor == "/", f"anchor = {Path('/a/b').anchor!r}"
assert Path("a/b").anchor == "", f"relative anchor = {Path('a/b').anchor!r}"
print("anchor_absolute_vs_relative OK")
"###);
    assert_output(&out, r###"anchor_absolute_vs_relative OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/equality_is_value_based.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_equality_is_value_based() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "equality_is_value_based"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path equality is value-based: Path('/tmp/a')==Path('/tmp/a') and Path('/tmp/a')!=Path('/tmp/b')"""
import pathlib

Path = pathlib.Path

assert Path("/tmp/a") == Path("/tmp/a"), "path equality"
assert Path("/tmp/a") != Path("/tmp/b"), "path inequality"
print("equality_is_value_based OK")
"###);
    assert_output(&out, r###"equality_is_value_based OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/glob_matches_suffix.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_glob_matches_suffix() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "glob_matches_suffix"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory holding a.txt/b.txt/c.py, glob('*.txt') returns exactly the two .txt entries and every result has suffix '.txt'"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _base_p = Path(_tmpdir)
    (_base_p / "a.txt").write_text("a")
    (_base_p / "b.txt").write_text("b")
    (_base_p / "c.py").write_text("c")
    _txts = sorted(_base_p.glob("*.txt"))
    assert len(_txts) == 2, f"glob *.txt count = {len(_txts)!r}"
    assert all(p.suffix == ".txt" for p in _txts), "all .txt"
print("glob_matches_suffix OK")
"###);
    assert_output(&out, r###"glob_matches_suffix OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/iterdir_lists_children.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_iterdir_lists_children() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "iterdir_lists_children"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory holding two files, list(iterdir()) yields exactly two entries"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _d = Path(_tmpdir)
    (_d / "x").write_text("x")
    (_d / "y").write_text("y")
    _items = list(_d.iterdir())
    assert len(_items) == 2, f"iterdir count = {len(_items)!r}"
print("iterdir_lists_children OK")
"###);
    assert_output(&out, r###"iterdir_lists_children OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/mkdir_rmdir_lifecycle.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_mkdir_rmdir_lifecycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "mkdir_rmdir_lifecycle"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory, mkdir() creates a dir that is_dir(), and rmdir() then removes it so it no longer exists()"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _subdir = Path(_tmpdir) / "newdir"
    _subdir.mkdir()
    assert _subdir.is_dir(), "mkdir creates dir"
    _subdir.rmdir()
    assert not _subdir.exists(), "rmdir removes dir"
print("mkdir_rmdir_lifecycle OK")
"###);
    assert_output(&out, r###"mkdir_rmdir_lifecycle OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/name_stem_suffix_parts.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_name_stem_suffix_parts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "name_stem_suffix_parts"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('/tmp/test/file.txt') exposes name='file.txt', stem='file', suffix='.txt', parent=Path('/tmp/test'), parts=('/','a','b','c'), and str() round-trips"""
import pathlib

Path = pathlib.Path

_p = Path("/tmp/test/file.txt")
assert isinstance(_p, Path), f"Path type = {type(_p)!r}"
assert _p.name == "file.txt", f"name = {_p.name!r}"
assert _p.stem == "file", f"stem = {_p.stem!r}"
assert _p.suffix == ".txt", f"suffix = {_p.suffix!r}"
assert _p.parent == Path("/tmp/test"), f"parent = {_p.parent!r}"

_parts = Path("/a/b/c").parts
assert _parts == ("/", "a", "b", "c"), f"parts = {_parts!r}"

assert str(Path("/tmp/foo")) == "/tmp/foo", f"str = {str(Path('/tmp/foo'))!r}"

print("name_stem_suffix_parts OK")
"###);
    assert_output(&out, r###"name_stem_suffix_parts OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_flavour_semantics.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_flavour_semantics() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_flavour_semantics"
# subject = "pathlib.PurePosixPath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath: POSIX-flavour rules: case sensitivity, single/double/triple-slash root handling, is_absolute, absolute-component reset on join, is_reserved always False, and coercion of a PureWindowsPath string form"""
import pathlib

P = pathlib.PurePosixPath
PureWindowsPath = pathlib.PureWindowsPath

# POSIX paths are case-sensitive.
assert P("a/b") != P("A/b"), "case-sensitive inequality"

# A single leading slash is one root; a lone double slash is a distinct
# "//" root (POSIX-implementation-defined), but three-or-more collapse to one.
assert P("/a").root == "/", "single-slash root"
assert P("//a").root == "//", f"double-slash root = {P('//a').root!r}"
assert P("///a").root == "/", "triple-slash collapses"
assert P("/a") == P("///a"), "/a == ///a"
assert P("/a") != P("//a"), "/a != //a (distinct roots)"

# is_absolute: any leading slash (including //) is absolute.
assert not P().is_absolute(), "empty is relative"
assert not P("a/b").is_absolute(), "relative path"
assert P("/a/b").is_absolute(), "leading slash is absolute"
assert P("//a/b").is_absolute(), "double-slash is absolute"

# Joining an absolute component discards everything before it.
assert P("/a") / "//c" == P("//c"), "// component resets"
assert P("//a") / "/c" == P("/c"), "/ component resets"
assert P("//a") / "b" == P("//a/b"), "double-slash root preserved on join"
assert P("/a").joinpath("//c") == P("//c"), "joinpath absolute reset"

# is_reserved is always False on POSIX, even for Windows-reserved-looking names.
assert P("").is_reserved() is False, "empty not reserved"
assert P("/dev/con/PRN/NUL").is_reserved() is False, "con/prn/nul not reserved"

# Constructing a POSIX path from a Windows pure path coerces its string form;
# a 'c:' becomes an ordinary path component (no drive on POSIX).
assert P("c:", "a", "b") == P(PureWindowsPath("c:\\a\\b")), "windows coercion"
print("posix_flavour_semantics OK")
"###);
    assert_output(&out, r###"posix_flavour_semantics OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_anchor_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_anchor_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_anchor_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_anchor_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_anchor_common
"""Auto-ported test: PosixPathAsPureTest::test_anchor_common (CPython 3.12 oracle)."""


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
sep = self_sep

assert P('').anchor == ''

assert P('a/b').anchor == ''

assert P('/').anchor == sep

assert P('/a/b').anchor == sep
print("PosixPathAsPureTest::test_anchor_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_anchor_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_as_posix_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_as_posix_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_as_posix_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_as_posix_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_as_posix_common
"""Auto-ported test: PosixPathAsPureTest::test_as_posix_common (CPython 3.12 oracle)."""


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
for pathstr in ('a', 'a/b', 'a/b/c', '/', '/a/b', '/a/b/c'):

    assert P(pathstr).as_posix() == pathstr
print("PosixPathAsPureTest::test_as_posix_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_as_posix_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_as_uri.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_as_uri() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_as_uri"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_as_uri"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_as_uri
"""Auto-ported test: PosixPathAsPureTest::test_as_uri (CPython 3.12 oracle)."""


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

assert P('/').as_uri() == 'file:///'

assert P('/a/b.c').as_uri() == 'file:///a/b.c'

assert P('/a/b%#c').as_uri() == 'file:///a/b%25%23c'
print("PosixPathAsPureTest::test_as_uri: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_as_uri: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_as_uri_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_as_uri_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_as_uri_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_as_uri_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_as_uri_common
"""Auto-ported test: PosixPathAsPureTest::test_as_uri_common (CPython 3.12 oracle)."""


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
try:
    P('a').as_uri()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    P().as_uri()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PosixPathAsPureTest::test_as_uri_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_as_uri_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_bytes"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_bytes
"""Auto-ported test: PosixPathAsPureTest::test_bytes (CPython 3.12 oracle)."""


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
message = "argument should be a str or an os\\.PathLike object where __fspath__ returns a str, not 'bytes'"
try:
    P(b'a')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(message, str(_aR_e))
try:
    P(b'a', 'b')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(message, str(_aR_e))
try:
    P('a', b'b')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(message, str(_aR_e))
try:
    P('a').joinpath(b'b')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    P('a') / b'b'
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    b'a' / P('b')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    P('a').match(b'b')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    P('a').relative_to(b'b')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    P('a').with_name(b'b')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    P('a').with_stem(b'b')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    P('a').with_suffix(b'b')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PosixPathAsPureTest::test_bytes: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_div.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_div() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_div"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_div"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_div
"""Auto-ported test: PosixPathAsPureTest::test_div (CPython 3.12 oracle)."""


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
p = P('//a')
pp = p / 'b'

assert pp == P('//a/b')
pp = P('/a') / '//c'

assert pp == P('//c')
pp = P('//a') / '/c'

assert pp == P('/c')
print("PosixPathAsPureTest::test_div: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_div: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_div_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_div_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_div_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_div_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_div_common
"""Auto-ported test: PosixPathAsPureTest::test_div_common (CPython 3.12 oracle)."""


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
p = P('a/b')
pp = p / 'c'

assert pp == P('a/b/c')

assert type(pp) is type(p)
pp = p / 'c/d'

assert pp == P('a/b/c/d')
pp = p / 'c' / 'd'

assert pp == P('a/b/c/d')
pp = 'c' / p / 'd'

assert pp == P('c/a/b/d')
pp = p / P('c')

assert pp == P('a/b/c')
pp = p / '/c'

assert pp == P('/c')
print("PosixPathAsPureTest::test_div_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_div_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_drive_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_drive_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_drive_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_drive_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_drive_common
"""Auto-ported test: PosixPathAsPureTest::test_drive_common (CPython 3.12 oracle)."""


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

assert P('a/b').drive == ''

assert P('/a/b').drive == ''

assert P('').drive == ''
print("PosixPathAsPureTest::test_drive_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_drive_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_eq.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_eq() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_eq"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_eq
"""Auto-ported test: PosixPathAsPureTest::test_eq (CPython 3.12 oracle)."""


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

assert P('a/b') != P('A/b')

assert P('/a') == P('///a')

assert P('/a') != P('//a')
print("PosixPathAsPureTest::test_eq: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_eq: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_eq_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_eq_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_eq_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_eq_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_eq_common
"""Auto-ported test: PosixPathAsPureTest::test_eq_common (CPython 3.12 oracle)."""


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

assert P('a/b') == P('a/b')

assert P('a/b') == P('a', 'b')

assert P('a/b') != P('a')

assert P('a/b') != P('/a/b')

assert P('a/b') != P()

assert P('/a/b') != P('/')

assert P() != P('/')

assert P() != ''

assert P() != {}

assert P() != int
print("PosixPathAsPureTest::test_eq_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_eq_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_equivalences.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_equivalences() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_equivalences"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_equivalences"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_equivalences
"""Auto-ported test: PosixPathAsPureTest::test_equivalences (CPython 3.12 oracle)."""


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
for k, tuples in equivalences.items():
    canon = k.replace('/', self_sep)
    posix = k.replace(self_sep, '/')
    if canon != posix:
        tuples = tuples + [tuple((part.replace('/', self_sep) for part in t)) for t in tuples]
        tuples.append((posix,))
    pcanon = cls(canon)
    for t in tuples:
        p = cls(*t)

        assert p == pcanon

        assert hash(p) == hash(pcanon)

        assert str(p) == canon

        assert p.as_posix() == posix
print("PosixPathAsPureTest::test_equivalences: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_equivalences: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_fspath_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_fspath_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_fspath_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_fspath_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_fspath_common
"""Auto-ported test: PosixPathAsPureTest::test_fspath_common (CPython 3.12 oracle)."""


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
p = P('a/b')
_check_str(p.__fspath__(), ('a/b',))
_check_str(os.fspath(p), ('a/b',))
print("PosixPathAsPureTest::test_fspath_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_fspath_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_is_absolute.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_is_absolute() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_is_absolute: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_is_reserved.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_is_reserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_is_reserved"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_is_reserved"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_is_reserved
"""Auto-ported test: PosixPathAsPureTest::test_is_reserved (CPython 3.12 oracle)."""


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

assert False is P('').is_reserved()

assert False is P('/').is_reserved()

assert False is P('/foo/bar').is_reserved()

assert False is P('/dev/con/PRN/NUL').is_reserved()
print("PosixPathAsPureTest::test_is_reserved: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_is_reserved: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_join.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_join() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_join"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_join
"""Auto-ported test: PosixPathAsPureTest::test_join (CPython 3.12 oracle)."""


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
p = P('//a')
pp = p.joinpath('b')

assert pp == P('//a/b')
pp = P('/a').joinpath('//c')

assert pp == P('//c')
pp = P('//a').joinpath('/c')

assert pp == P('/c')
print("PosixPathAsPureTest::test_join: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_join: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_join_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_join_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_join_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_join_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_join_common
"""Auto-ported test: PosixPathAsPureTest::test_join_common (CPython 3.12 oracle)."""


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
p = P('a/b')
pp = p.joinpath('c')

assert pp == P('a/b/c')

assert type(pp) is type(p)
pp = p.joinpath('c', 'd')

assert pp == P('a/b/c/d')
pp = p.joinpath(P('c'))

assert pp == P('a/b/c')
pp = p.joinpath('/c')

assert pp == P('/c')
print("PosixPathAsPureTest::test_join_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_join_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_match.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_match"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_match
"""Auto-ported test: PosixPathAsPureTest::test_match (CPython 3.12 oracle)."""


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

assert not P('A.py').match('a.PY')
print("PosixPathAsPureTest::test_match: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_match: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_match_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_match_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_match_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_match_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_match_common
"""Auto-ported test: PosixPathAsPureTest::test_match_common (CPython 3.12 oracle)."""


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
print("PosixPathAsPureTest::test_match_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_match_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_name_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_name_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_name_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_name_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_name_common
"""Auto-ported test: PosixPathAsPureTest::test_name_common (CPython 3.12 oracle)."""


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

assert P('').name == ''

assert P('.').name == ''

assert P('/').name == ''

assert P('a/b').name == 'b'

assert P('/a/b').name == 'b'

assert P('/a/b/.').name == 'b'

assert P('a/b.py').name == 'b.py'

assert P('/a/b.py').name == 'b.py'
print("PosixPathAsPureTest::test_name_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_name_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_parent_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_parent_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_parent_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_parent_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_parent_common
"""Auto-ported test: PosixPathAsPureTest::test_parent_common (CPython 3.12 oracle)."""


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
p = P('a/b/c')

assert p.parent == P('a/b')

assert p.parent.parent == P('a')

assert p.parent.parent.parent == P()

assert p.parent.parent.parent.parent == P()
p = P('/a/b/c')

assert p.parent == P('/a/b')

assert p.parent.parent == P('/a')

assert p.parent.parent.parent == P('/')

assert p.parent.parent.parent.parent == P('/')
print("PosixPathAsPureTest::test_parent_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_parent_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_parse_windows_path.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_parse_windows_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_parse_windows_path"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_parse_windows_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_parse_windows_path
"""Auto-ported test: PosixPathAsPureTest::test_parse_windows_path (CPython 3.12 oracle)."""


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
p = P('c:', 'a', 'b')
pp = P(pathlib.PureWindowsPath('c:\\a\\b'))

assert p == pp
print("PosixPathAsPureTest::test_parse_windows_path: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_parse_windows_path: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_parts_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_parts_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_parts_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_parts_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_parts_common
"""Auto-ported test: PosixPathAsPureTest::test_parts_common (CPython 3.12 oracle)."""


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
sep = self_sep
P = cls
p = P('a/b')
parts = p.parts

assert parts == ('a', 'b')
p = P('/a/b')
parts = p.parts

assert parts == (sep, 'a', 'b')
print("PosixPathAsPureTest::test_parts_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_parts_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_pickling_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_pickling_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_pickling_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_pickling_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_pickling_common
"""Auto-ported test: PosixPathAsPureTest::test_pickling_common (CPython 3.12 oracle)."""


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
p = P('/a/b')
for proto in range(0, pickle.HIGHEST_PROTOCOL + 1):
    dumped = pickle.dumps(p, proto)
    pp = pickle.loads(dumped)

    assert pp.__class__ is p.__class__

    assert pp == p

    assert hash(pp) == hash(p)

    assert str(pp) == str(p)
print("PosixPathAsPureTest::test_pickling_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_pickling_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_root.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_root() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_root"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_root"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_root
"""Auto-ported test: PosixPathAsPureTest::test_root (CPython 3.12 oracle)."""


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

assert P('/a/b').root == '/'

assert P('///a/b').root == '/'

assert P('//a/b').root == '//'
print("PosixPathAsPureTest::test_root: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_root: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_root_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_root_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_root_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_root_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_root_common
"""Auto-ported test: PosixPathAsPureTest::test_root_common (CPython 3.12 oracle)."""


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
sep = self_sep

assert P('').root == ''

assert P('a/b').root == ''

assert P('/').root == sep

assert P('/a/b').root == sep
print("PosixPathAsPureTest::test_root_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_root_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_stem_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_stem_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_stem_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_stem_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_stem_common
"""Auto-ported test: PosixPathAsPureTest::test_stem_common (CPython 3.12 oracle)."""


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

assert P('').stem == ''

assert P('.').stem == ''

assert P('..').stem == '..'

assert P('/').stem == ''

assert P('a/b').stem == 'b'

assert P('a/b.py').stem == 'b'

assert P('a/.hgrc').stem == '.hgrc'

assert P('a/.hg.rc').stem == '.hg'

assert P('a/b.tar.gz').stem == 'b.tar'

assert P('a/Some name. Ending with a dot.').stem == 'Some name. Ending with a dot.'
print("PosixPathAsPureTest::test_stem_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_stem_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_str_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_str_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_str_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_str_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_str_common
"""Auto-ported test: PosixPathAsPureTest::test_str_common (CPython 3.12 oracle)."""


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
for pathstr in ('a', 'a/b', 'a/b/c', '/', '/a/b', '/a/b/c'):
    _check_str(pathstr, (pathstr,))
_check_str('.', ('',))
print("PosixPathAsPureTest::test_str_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_str_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_suffix_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_suffix_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_suffix_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_suffix_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_suffix_common
"""Auto-ported test: PosixPathAsPureTest::test_suffix_common (CPython 3.12 oracle)."""


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

assert P('').suffix == ''

assert P('.').suffix == ''

assert P('..').suffix == ''

assert P('/').suffix == ''

assert P('a/b').suffix == ''

assert P('/a/b').suffix == ''

assert P('/a/b/.').suffix == ''

assert P('a/b.py').suffix == '.py'

assert P('/a/b.py').suffix == '.py'

assert P('a/.hgrc').suffix == ''

assert P('/a/.hgrc').suffix == ''

assert P('a/.hg.rc').suffix == '.rc'

assert P('/a/.hg.rc').suffix == '.rc'

assert P('a/b.tar.gz').suffix == '.gz'

assert P('/a/b.tar.gz').suffix == '.gz'

assert P('a/Some name. Ending with a dot.').suffix == ''

assert P('/a/Some name. Ending with a dot.').suffix == ''
print("PosixPathAsPureTest::test_suffix_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_suffix_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_suffixes_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_suffixes_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_suffixes_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_suffixes_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_suffixes_common
"""Auto-ported test: PosixPathAsPureTest::test_suffixes_common (CPython 3.12 oracle)."""


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

assert P('').suffixes == []

assert P('.').suffixes == []

assert P('/').suffixes == []

assert P('a/b').suffixes == []

assert P('/a/b').suffixes == []

assert P('/a/b/.').suffixes == []

assert P('a/b.py').suffixes == ['.py']

assert P('/a/b.py').suffixes == ['.py']

assert P('a/.hgrc').suffixes == []

assert P('/a/.hgrc').suffixes == []

assert P('a/.hg.rc').suffixes == ['.rc']

assert P('/a/.hg.rc').suffixes == ['.rc']

assert P('a/b.tar.gz').suffixes == ['.tar', '.gz']

assert P('/a/b.tar.gz').suffixes == ['.tar', '.gz']

assert P('a/Some name. Ending with a dot.').suffixes == []

assert P('/a/Some name. Ending with a dot.').suffixes == []
print("PosixPathAsPureTest::test_suffixes_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_suffixes_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_with_name_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_with_name_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_with_name_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_with_name_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_with_name_common
"""Auto-ported test: PosixPathAsPureTest::test_with_name_common (CPython 3.12 oracle)."""


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

assert P('a/b').with_name('d.xml') == P('a/d.xml')

assert P('/a/b').with_name('d.xml') == P('/a/d.xml')

assert P('a/b.py').with_name('d.xml') == P('a/d.xml')

assert P('/a/b.py').with_name('d.xml') == P('/a/d.xml')

assert P('a/Dot ending.').with_name('d.xml') == P('a/d.xml')

assert P('/a/Dot ending.').with_name('d.xml') == P('/a/d.xml')

try:
    P('').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('.').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('/').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('/c')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('c/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('c/d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PosixPathAsPureTest::test_with_name_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_with_name_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_with_stem_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_with_stem_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_with_stem_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_with_stem_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_with_stem_common
"""Auto-ported test: PosixPathAsPureTest::test_with_stem_common (CPython 3.12 oracle)."""


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

assert P('a/b').with_stem('d') == P('a/d')

assert P('/a/b').with_stem('d') == P('/a/d')

assert P('a/b.py').with_stem('d') == P('a/d.py')

assert P('/a/b.py').with_stem('d') == P('/a/d.py')

assert P('/a/b.tar.gz').with_stem('d') == P('/a/d.gz')

assert P('a/Dot ending.').with_stem('d') == P('a/d')

assert P('/a/Dot ending.').with_stem('d') == P('/a/d')

try:
    P('').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('.').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('/').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('/c')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('c/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('c/d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PosixPathAsPureTest::test_with_stem_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_with_stem_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/posix_path_as_pure_test__test_with_suffix_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_posix_path_as_pure_test__test_with_suffix_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_as_pure_test__test_with_suffix_common"
# subject = "cpython.test_pathlib.PosixPathAsPureTest.test_with_suffix_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PosixPathAsPureTest::test_with_suffix_common
"""Auto-ported test: PosixPathAsPureTest::test_with_suffix_common (CPython 3.12 oracle)."""


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

assert P('a/b').with_suffix('.gz') == P('a/b.gz')

assert P('/a/b').with_suffix('.gz') == P('/a/b.gz')

assert P('a/b.py').with_suffix('.gz') == P('a/b.gz')

assert P('/a/b.py').with_suffix('.gz') == P('/a/b.gz')

assert P('a/b.py').with_suffix('') == P('a/b')

assert P('/a/b').with_suffix('') == P('/a/b')

try:
    P('').with_suffix('.gz')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('.').with_suffix('.gz')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('/').with_suffix('.gz')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('gz')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('/.gz')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('c/d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('.c/.d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('./.d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix('.d/.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_suffix((self_flavour.sep, 'd'))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PosixPathAsPureTest::test_with_suffix_common: ok")
"###);
    assert_output(&out, r###"PosixPathAsPureTest::test_with_suffix_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_parsing.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_parsing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_parsing"
# subject = "pathlib.PurePath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath: PurePath parsing: anchor/root/drive on absolute vs relative, multi-arg and PurePath-instance constructors, the parent chain bottoming out at '.'/'/', the parents sequence (index/negative/slice/iterate/immutable), and name/stem/suffix dot edge cases"""
import pathlib

P = pathlib.PurePath

# Anchor / root / drive on POSIX-style paths.
assert P("").anchor == "", "empty anchor"
assert P("a/b").anchor == "", "relative anchor"
assert P("/a/b").anchor == "/", f"absolute anchor = {P('/a/b').anchor!r}"
assert P("/a/b").root == "/", "absolute root"
assert P("a/b").root == "", "relative root"
assert P("/a/b").drive == "", "posix has no drive"

# Constructor accepts multiple args and PurePath instances.
assert P("a", "b", "c") == P("a/b/c"), "multi-arg join"
assert P(P("a"), "b") == P("a/b"), "PurePath + str"
assert P(P("a"), P("b"), P("c")) == P("a/b/c"), "PurePath chain"
assert P(P("a")) == P("a"), "PurePath copy"

# Parent chain bottoms out at '.' (relative) or '/' (absolute).
_rel = P("a/b/c")
assert _rel.parent == P("a/b"), "rel parent"
assert _rel.parent.parent.parent == P("."), "rel parent floor is ."
assert _rel.parent.parent.parent.parent == P("."), "rel parent floor sticks"
_abs = P("/a/b/c")
assert _abs.parent.parent.parent == P("/"), "abs parent floor is /"
assert _abs.parent.parent.parent.parent == P("/"), "abs parent floor sticks"

# parents is an indexable sequence with negative indexing and slicing.
_par = P("a/b/c").parents
assert len(_par) == 3, f"parents len = {len(_par)!r}"
assert _par[0] == P("a/b"), "parents[0]"
assert _par[2] == P("."), "parents[-1] is ."
assert _par[-1] == P("."), "negative index"
assert _par[:2] == (P("a/b"), P("a")), "slice"
assert _par[::-1] == (P("."), P("a"), P("a/b")), "reverse slice"
assert list(_par) == [P("a/b"), P("a"), P(".")], "iterate"

# Out-of-range index raises IndexError; the sequence is immutable.
try:
    _par[3]
    raise AssertionError("parents[3] should raise IndexError")
except IndexError:
    pass
try:
    _par[0] = P("x")  # type: ignore[index]
    raise AssertionError("parents assignment should raise TypeError")
except TypeError:
    pass

# name/stem/suffix edge cases around dots.
assert P("..").stem == "..", "double-dot stem"
assert P("a/.hgrc").stem == ".hgrc", "dotfile stem"
assert P("a/.hg.rc").suffix == ".rc", "dotfile suffix"
assert P("a/Dot ending.").suffix == "", "trailing dot has no suffix"
assert P("/a/b/.").name == "b", "trailing /. ignored for name"
print("pure_path_parsing OK")
"###);
    assert_output(&out, r###"pure_path_parsing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_anchor_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_anchor_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_anchor_common"
# subject = "cpython.test_pathlib.PurePathTest.test_anchor_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_anchor_common
"""Auto-ported test: PurePathTest::test_anchor_common (CPython 3.12 oracle)."""


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
sep = self_sep

assert P('').anchor == ''

assert P('a/b').anchor == ''

assert P('/').anchor == sep

assert P('/a/b').anchor == sep
print("PurePathTest::test_anchor_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_anchor_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_as_posix_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_as_posix_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_as_posix_common"
# subject = "cpython.test_pathlib.PurePathTest.test_as_posix_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_as_posix_common
"""Auto-ported test: PurePathTest::test_as_posix_common (CPython 3.12 oracle)."""


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
for pathstr in ('a', 'a/b', 'a/b/c', '/', '/a/b', '/a/b/c'):

    assert P(pathstr).as_posix() == pathstr
print("PurePathTest::test_as_posix_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_as_posix_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_as_uri_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_as_uri_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_as_uri_common"
# subject = "cpython.test_pathlib.PurePathTest.test_as_uri_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_as_uri_common
"""Auto-ported test: PurePathTest::test_as_uri_common (CPython 3.12 oracle)."""


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
    P('a').as_uri()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    P().as_uri()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PurePathTest::test_as_uri_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_as_uri_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_different_flavours_unequal.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_different_flavours_unequal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_different_flavours_unequal"
# subject = "cpython.test_pathlib.PurePathTest.test_different_flavours_unequal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_different_flavours_unequal
"""Auto-ported test: PurePathTest::test_different_flavours_unequal (CPython 3.12 oracle)."""


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
p = pathlib.PurePosixPath('a')
q = pathlib.PureWindowsPath('a')

assert p != q
print("PurePathTest::test_different_flavours_unequal: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_different_flavours_unequal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_different_flavours_unordered.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_different_flavours_unordered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_different_flavours_unordered"
# subject = "cpython.test_pathlib.PurePathTest.test_different_flavours_unordered"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_different_flavours_unordered
"""Auto-ported test: PurePathTest::test_different_flavours_unordered (CPython 3.12 oracle)."""


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
p = pathlib.PurePosixPath('a')
q = pathlib.PureWindowsPath('a')
try:
    p < q
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    p <= q
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    p > q
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    p >= q
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PurePathTest::test_different_flavours_unordered: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_different_flavours_unordered: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_div_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_div_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_div_common"
# subject = "cpython.test_pathlib.PurePathTest.test_div_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_div_common
"""Auto-ported test: PurePathTest::test_div_common (CPython 3.12 oracle)."""


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
p = P('a/b')
pp = p / 'c'

assert pp == P('a/b/c')

assert type(pp) is type(p)
pp = p / 'c/d'

assert pp == P('a/b/c/d')
pp = p / 'c' / 'd'

assert pp == P('a/b/c/d')
pp = 'c' / p / 'd'

assert pp == P('c/a/b/d')
pp = p / P('c')

assert pp == P('a/b/c')
pp = p / '/c'

assert pp == P('/c')
print("PurePathTest::test_div_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_div_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_drive_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_drive_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_drive_common"
# subject = "cpython.test_pathlib.PurePathTest.test_drive_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_drive_common
"""Auto-ported test: PurePathTest::test_drive_common (CPython 3.12 oracle)."""


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

assert P('a/b').drive == ''

assert P('/a/b').drive == ''

assert P('').drive == ''
print("PurePathTest::test_drive_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_drive_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_eq_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_eq_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_eq_common"
# subject = "cpython.test_pathlib.PurePathTest.test_eq_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_eq_common
"""Auto-ported test: PurePathTest::test_eq_common (CPython 3.12 oracle)."""


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

assert P('a/b') == P('a/b')

assert P('a/b') == P('a', 'b')

assert P('a/b') != P('a')

assert P('a/b') != P('/a/b')

assert P('a/b') != P()

assert P('/a/b') != P('/')

assert P() != P('/')

assert P() != ''

assert P() != {}

assert P() != int
print("PurePathTest::test_eq_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_eq_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_equivalences.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_equivalences() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_equivalences"
# subject = "cpython.test_pathlib.PurePathTest.test_equivalences"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_equivalences
"""Auto-ported test: PurePathTest::test_equivalences (CPython 3.12 oracle)."""


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
for k, tuples in equivalences.items():
    canon = k.replace('/', self_sep)
    posix = k.replace(self_sep, '/')
    if canon != posix:
        tuples = tuples + [tuple((part.replace('/', self_sep) for part in t)) for t in tuples]
        tuples.append((posix,))
    pcanon = cls(canon)
    for t in tuples:
        p = cls(*t)

        assert p == pcanon

        assert hash(p) == hash(pcanon)

        assert str(p) == canon

        assert p.as_posix() == posix
print("PurePathTest::test_equivalences: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_equivalences: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_fspath_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_fspath_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_fspath_common"
# subject = "cpython.test_pathlib.PurePathTest.test_fspath_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_fspath_common
"""Auto-ported test: PurePathTest::test_fspath_common (CPython 3.12 oracle)."""


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
p = P('a/b')
_check_str(p.__fspath__(), ('a/b',))
_check_str(os.fspath(p), ('a/b',))
print("PurePathTest::test_fspath_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_fspath_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_join_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_join_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_join_common"
# subject = "cpython.test_pathlib.PurePathTest.test_join_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_join_common
"""Auto-ported test: PurePathTest::test_join_common (CPython 3.12 oracle)."""


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
p = P('a/b')
pp = p.joinpath('c')

assert pp == P('a/b/c')

assert type(pp) is type(p)
pp = p.joinpath('c', 'd')

assert pp == P('a/b/c/d')
pp = p.joinpath(P('c'))

assert pp == P('a/b/c')
pp = p.joinpath('/c')

assert pp == P('/c')
print("PurePathTest::test_join_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_join_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_match_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_match_common() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"PurePathTest::test_match_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_name_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_name_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_name_common"
# subject = "cpython.test_pathlib.PurePathTest.test_name_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_name_common
"""Auto-ported test: PurePathTest::test_name_common (CPython 3.12 oracle)."""


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

assert P('').name == ''

assert P('.').name == ''

assert P('/').name == ''

assert P('a/b').name == 'b'

assert P('/a/b').name == 'b'

assert P('/a/b/.').name == 'b'

assert P('a/b.py').name == 'b.py'

assert P('/a/b.py').name == 'b.py'
print("PurePathTest::test_name_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_name_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_parent_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_parent_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_parent_common"
# subject = "cpython.test_pathlib.PurePathTest.test_parent_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_parent_common
"""Auto-ported test: PurePathTest::test_parent_common (CPython 3.12 oracle)."""


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
p = P('a/b/c')

assert p.parent == P('a/b')

assert p.parent.parent == P('a')

assert p.parent.parent.parent == P()

assert p.parent.parent.parent.parent == P()
p = P('/a/b/c')

assert p.parent == P('/a/b')

assert p.parent.parent == P('/a')

assert p.parent.parent.parent == P('/')

assert p.parent.parent.parent.parent == P('/')
print("PurePathTest::test_parent_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_parent_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_parts_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_parts_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_parts_common"
# subject = "cpython.test_pathlib.PurePathTest.test_parts_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_parts_common
"""Auto-ported test: PurePathTest::test_parts_common (CPython 3.12 oracle)."""


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
sep = self_sep
P = cls
p = P('a/b')
parts = p.parts

assert parts == ('a', 'b')
p = P('/a/b')
parts = p.parts

assert parts == (sep, 'a', 'b')
print("PurePathTest::test_parts_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_parts_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_pickling_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_pickling_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_pickling_common"
# subject = "cpython.test_pathlib.PurePathTest.test_pickling_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_pickling_common
"""Auto-ported test: PurePathTest::test_pickling_common (CPython 3.12 oracle)."""


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
p = P('/a/b')
for proto in range(0, pickle.HIGHEST_PROTOCOL + 1):
    dumped = pickle.dumps(p, proto)
    pp = pickle.loads(dumped)

    assert pp.__class__ is p.__class__

    assert pp == p

    assert hash(pp) == hash(p)

    assert str(pp) == str(p)
print("PurePathTest::test_pickling_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_pickling_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_root_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_root_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_root_common"
# subject = "cpython.test_pathlib.PurePathTest.test_root_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_root_common
"""Auto-ported test: PurePathTest::test_root_common (CPython 3.12 oracle)."""


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
sep = self_sep

assert P('').root == ''

assert P('a/b').root == ''

assert P('/').root == sep

assert P('/a/b').root == sep
print("PurePathTest::test_root_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_root_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_stem_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_stem_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_stem_common"
# subject = "cpython.test_pathlib.PurePathTest.test_stem_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_stem_common
"""Auto-ported test: PurePathTest::test_stem_common (CPython 3.12 oracle)."""


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

assert P('').stem == ''

assert P('.').stem == ''

assert P('..').stem == '..'

assert P('/').stem == ''

assert P('a/b').stem == 'b'

assert P('a/b.py').stem == 'b'

assert P('a/.hgrc').stem == '.hgrc'

assert P('a/.hg.rc').stem == '.hg'

assert P('a/b.tar.gz').stem == 'b.tar'

assert P('a/Some name. Ending with a dot.').stem == 'Some name. Ending with a dot.'
print("PurePathTest::test_stem_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_stem_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_str_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_str_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_str_common"
# subject = "cpython.test_pathlib.PurePathTest.test_str_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_str_common
"""Auto-ported test: PurePathTest::test_str_common (CPython 3.12 oracle)."""


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
for pathstr in ('a', 'a/b', 'a/b/c', '/', '/a/b', '/a/b/c'):
    _check_str(pathstr, (pathstr,))
_check_str('.', ('',))
print("PurePathTest::test_str_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_str_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_suffix_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_suffix_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_suffix_common"
# subject = "cpython.test_pathlib.PurePathTest.test_suffix_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_suffix_common
"""Auto-ported test: PurePathTest::test_suffix_common (CPython 3.12 oracle)."""


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

assert P('').suffix == ''

assert P('.').suffix == ''

assert P('..').suffix == ''

assert P('/').suffix == ''

assert P('a/b').suffix == ''

assert P('/a/b').suffix == ''

assert P('/a/b/.').suffix == ''

assert P('a/b.py').suffix == '.py'

assert P('/a/b.py').suffix == '.py'

assert P('a/.hgrc').suffix == ''

assert P('/a/.hgrc').suffix == ''

assert P('a/.hg.rc').suffix == '.rc'

assert P('/a/.hg.rc').suffix == '.rc'

assert P('a/b.tar.gz').suffix == '.gz'

assert P('/a/b.tar.gz').suffix == '.gz'

assert P('a/Some name. Ending with a dot.').suffix == ''

assert P('/a/Some name. Ending with a dot.').suffix == ''
print("PurePathTest::test_suffix_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_suffix_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_suffixes_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_suffixes_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_suffixes_common"
# subject = "cpython.test_pathlib.PurePathTest.test_suffixes_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_suffixes_common
"""Auto-ported test: PurePathTest::test_suffixes_common (CPython 3.12 oracle)."""


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

assert P('').suffixes == []

assert P('.').suffixes == []

assert P('/').suffixes == []

assert P('a/b').suffixes == []

assert P('/a/b').suffixes == []

assert P('/a/b/.').suffixes == []

assert P('a/b.py').suffixes == ['.py']

assert P('/a/b.py').suffixes == ['.py']

assert P('a/.hgrc').suffixes == []

assert P('/a/.hgrc').suffixes == []

assert P('a/.hg.rc').suffixes == ['.rc']

assert P('/a/.hg.rc').suffixes == ['.rc']

assert P('a/b.tar.gz').suffixes == ['.tar', '.gz']

assert P('/a/b.tar.gz').suffixes == ['.tar', '.gz']

assert P('a/Some name. Ending with a dot.').suffixes == []

assert P('/a/Some name. Ending with a dot.').suffixes == []
print("PurePathTest::test_suffixes_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_suffixes_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_with_name_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_with_name_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_with_name_common"
# subject = "cpython.test_pathlib.PurePathTest.test_with_name_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_with_name_common
"""Auto-ported test: PurePathTest::test_with_name_common (CPython 3.12 oracle)."""


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

assert P('a/b').with_name('d.xml') == P('a/d.xml')

assert P('/a/b').with_name('d.xml') == P('/a/d.xml')

assert P('a/b.py').with_name('d.xml') == P('a/d.xml')

assert P('/a/b.py').with_name('d.xml') == P('/a/d.xml')

assert P('a/Dot ending.').with_name('d.xml') == P('a/d.xml')

assert P('/a/Dot ending.').with_name('d.xml') == P('/a/d.xml')

try:
    P('').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('.').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('/').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('/c')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('c/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('c/d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PurePathTest::test_with_name_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_with_name_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_path_test__test_with_stem_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_path_test__test_with_stem_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_test__test_with_stem_common"
# subject = "cpython.test_pathlib.PurePathTest.test_with_stem_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePathTest::test_with_stem_common
"""Auto-ported test: PurePathTest::test_with_stem_common (CPython 3.12 oracle)."""


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

assert P('a/b').with_stem('d') == P('a/d')

assert P('/a/b').with_stem('d') == P('/a/d')

assert P('a/b.py').with_stem('d') == P('a/d.py')

assert P('/a/b.py').with_stem('d') == P('/a/d.py')

assert P('/a/b.tar.gz').with_stem('d') == P('/a/d.gz')

assert P('a/Dot ending.').with_stem('d') == P('a/d')

assert P('/a/Dot ending.').with_stem('d') == P('/a/d')

try:
    P('').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('.').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('/').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('/c')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('c/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('c/d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PurePathTest::test_with_stem_common: ok")
"###);
    assert_output(&out, r###"PurePathTest::test_with_stem_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_anchor_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_anchor_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_anchor_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_anchor_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_anchor_common
"""Auto-ported test: PurePosixPathTest::test_anchor_common (CPython 3.12 oracle)."""


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
sep = self_sep

assert P('').anchor == ''

assert P('a/b').anchor == ''

assert P('/').anchor == sep

assert P('/a/b').anchor == sep
print("PurePosixPathTest::test_anchor_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_anchor_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_as_posix_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_as_posix_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_as_posix_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_as_posix_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_as_posix_common
"""Auto-ported test: PurePosixPathTest::test_as_posix_common (CPython 3.12 oracle)."""


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
for pathstr in ('a', 'a/b', 'a/b/c', '/', '/a/b', '/a/b/c'):

    assert P(pathstr).as_posix() == pathstr
print("PurePosixPathTest::test_as_posix_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_as_posix_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_as_uri.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_as_uri() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_as_uri"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_as_uri"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_as_uri
"""Auto-ported test: PurePosixPathTest::test_as_uri (CPython 3.12 oracle)."""


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

assert P('/').as_uri() == 'file:///'

assert P('/a/b.c').as_uri() == 'file:///a/b.c'

assert P('/a/b%#c').as_uri() == 'file:///a/b%25%23c'
print("PurePosixPathTest::test_as_uri: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_as_uri: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_as_uri_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_as_uri_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_as_uri_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_as_uri_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_as_uri_common
"""Auto-ported test: PurePosixPathTest::test_as_uri_common (CPython 3.12 oracle)."""


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
    P('a').as_uri()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    P().as_uri()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PurePosixPathTest::test_as_uri_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_as_uri_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_div.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_div() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_div"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_div"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_div
"""Auto-ported test: PurePosixPathTest::test_div (CPython 3.12 oracle)."""


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
p = P('//a')
pp = p / 'b'

assert pp == P('//a/b')
pp = P('/a') / '//c'

assert pp == P('//c')
pp = P('//a') / '/c'

assert pp == P('/c')
print("PurePosixPathTest::test_div: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_div: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_div_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_div_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_div_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_div_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_div_common
"""Auto-ported test: PurePosixPathTest::test_div_common (CPython 3.12 oracle)."""


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
p = P('a/b')
pp = p / 'c'

assert pp == P('a/b/c')

assert type(pp) is type(p)
pp = p / 'c/d'

assert pp == P('a/b/c/d')
pp = p / 'c' / 'd'

assert pp == P('a/b/c/d')
pp = 'c' / p / 'd'

assert pp == P('c/a/b/d')
pp = p / P('c')

assert pp == P('a/b/c')
pp = p / '/c'

assert pp == P('/c')
print("PurePosixPathTest::test_div_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_div_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_drive_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_drive_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_drive_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_drive_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_drive_common
"""Auto-ported test: PurePosixPathTest::test_drive_common (CPython 3.12 oracle)."""


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

assert P('a/b').drive == ''

assert P('/a/b').drive == ''

assert P('').drive == ''
print("PurePosixPathTest::test_drive_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_drive_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_eq.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_eq() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_eq"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_eq
"""Auto-ported test: PurePosixPathTest::test_eq (CPython 3.12 oracle)."""


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

assert P('a/b') != P('A/b')

assert P('/a') == P('///a')

assert P('/a') != P('//a')
print("PurePosixPathTest::test_eq: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_eq: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_eq_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_eq_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_eq_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_eq_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_eq_common
"""Auto-ported test: PurePosixPathTest::test_eq_common (CPython 3.12 oracle)."""


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

assert P('a/b') == P('a/b')

assert P('a/b') == P('a', 'b')

assert P('a/b') != P('a')

assert P('a/b') != P('/a/b')

assert P('a/b') != P()

assert P('/a/b') != P('/')

assert P() != P('/')

assert P() != ''

assert P() != {}

assert P() != int
print("PurePosixPathTest::test_eq_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_eq_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_equivalences.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_equivalences() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_equivalences"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_equivalences"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_equivalences
"""Auto-ported test: PurePosixPathTest::test_equivalences (CPython 3.12 oracle)."""


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
for k, tuples in equivalences.items():
    canon = k.replace('/', self_sep)
    posix = k.replace(self_sep, '/')
    if canon != posix:
        tuples = tuples + [tuple((part.replace('/', self_sep) for part in t)) for t in tuples]
        tuples.append((posix,))
    pcanon = cls(canon)
    for t in tuples:
        p = cls(*t)

        assert p == pcanon

        assert hash(p) == hash(pcanon)

        assert str(p) == canon

        assert p.as_posix() == posix
print("PurePosixPathTest::test_equivalences: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_equivalences: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_fspath_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_fspath_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_fspath_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_fspath_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_fspath_common
"""Auto-ported test: PurePosixPathTest::test_fspath_common (CPython 3.12 oracle)."""


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
p = P('a/b')
_check_str(p.__fspath__(), ('a/b',))
_check_str(os.fspath(p), ('a/b',))
print("PurePosixPathTest::test_fspath_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_fspath_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_is_absolute.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_is_absolute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_is_absolute"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_is_absolute"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_is_absolute
"""Auto-ported test: PurePosixPathTest::test_is_absolute (CPython 3.12 oracle)."""


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
print("PurePosixPathTest::test_is_absolute: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_is_absolute: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_is_reserved.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_is_reserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_is_reserved"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_is_reserved"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_is_reserved
"""Auto-ported test: PurePosixPathTest::test_is_reserved (CPython 3.12 oracle)."""


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

assert False is P('').is_reserved()

assert False is P('/').is_reserved()

assert False is P('/foo/bar').is_reserved()

assert False is P('/dev/con/PRN/NUL').is_reserved()
print("PurePosixPathTest::test_is_reserved: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_is_reserved: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_join.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_join() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_join"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_join
"""Auto-ported test: PurePosixPathTest::test_join (CPython 3.12 oracle)."""


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
p = P('//a')
pp = p.joinpath('b')

assert pp == P('//a/b')
pp = P('/a').joinpath('//c')

assert pp == P('//c')
pp = P('//a').joinpath('/c')

assert pp == P('/c')
print("PurePosixPathTest::test_join: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_join: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_join_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_join_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_join_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_join_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_join_common
"""Auto-ported test: PurePosixPathTest::test_join_common (CPython 3.12 oracle)."""


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
p = P('a/b')
pp = p.joinpath('c')

assert pp == P('a/b/c')

assert type(pp) is type(p)
pp = p.joinpath('c', 'd')

assert pp == P('a/b/c/d')
pp = p.joinpath(P('c'))

assert pp == P('a/b/c')
pp = p.joinpath('/c')

assert pp == P('/c')
print("PurePosixPathTest::test_join_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_join_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_match.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_match() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_match"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_match
"""Auto-ported test: PurePosixPathTest::test_match (CPython 3.12 oracle)."""


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

assert not P('A.py').match('a.PY')
print("PurePosixPathTest::test_match: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_match: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_match_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_match_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_match_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_match_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_match_common
"""Auto-ported test: PurePosixPathTest::test_match_common (CPython 3.12 oracle)."""


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
print("PurePosixPathTest::test_match_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_match_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_name_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_name_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_name_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_name_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_name_common
"""Auto-ported test: PurePosixPathTest::test_name_common (CPython 3.12 oracle)."""


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

assert P('').name == ''

assert P('.').name == ''

assert P('/').name == ''

assert P('a/b').name == 'b'

assert P('/a/b').name == 'b'

assert P('/a/b/.').name == 'b'

assert P('a/b.py').name == 'b.py'

assert P('/a/b.py').name == 'b.py'
print("PurePosixPathTest::test_name_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_name_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_parent_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_parent_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_parent_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_parent_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_parent_common
"""Auto-ported test: PurePosixPathTest::test_parent_common (CPython 3.12 oracle)."""


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
p = P('a/b/c')

assert p.parent == P('a/b')

assert p.parent.parent == P('a')

assert p.parent.parent.parent == P()

assert p.parent.parent.parent.parent == P()
p = P('/a/b/c')

assert p.parent == P('/a/b')

assert p.parent.parent == P('/a')

assert p.parent.parent.parent == P('/')

assert p.parent.parent.parent.parent == P('/')
print("PurePosixPathTest::test_parent_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_parent_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_parse_windows_path.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_parse_windows_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_parse_windows_path"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_parse_windows_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_parse_windows_path
"""Auto-ported test: PurePosixPathTest::test_parse_windows_path (CPython 3.12 oracle)."""


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
p = P('c:', 'a', 'b')
pp = P(pathlib.PureWindowsPath('c:\\a\\b'))

assert p == pp
print("PurePosixPathTest::test_parse_windows_path: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_parse_windows_path: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_parts_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_parts_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_parts_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_parts_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_parts_common
"""Auto-ported test: PurePosixPathTest::test_parts_common (CPython 3.12 oracle)."""


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
sep = self_sep
P = cls
p = P('a/b')
parts = p.parts

assert parts == ('a', 'b')
p = P('/a/b')
parts = p.parts

assert parts == (sep, 'a', 'b')
print("PurePosixPathTest::test_parts_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_parts_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_pickling_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_pickling_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_pickling_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_pickling_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_pickling_common
"""Auto-ported test: PurePosixPathTest::test_pickling_common (CPython 3.12 oracle)."""


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
p = P('/a/b')
for proto in range(0, pickle.HIGHEST_PROTOCOL + 1):
    dumped = pickle.dumps(p, proto)
    pp = pickle.loads(dumped)

    assert pp.__class__ is p.__class__

    assert pp == p

    assert hash(pp) == hash(p)

    assert str(pp) == str(p)
print("PurePosixPathTest::test_pickling_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_pickling_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_root.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_root() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_root"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_root"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_root
"""Auto-ported test: PurePosixPathTest::test_root (CPython 3.12 oracle)."""


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

assert P('/a/b').root == '/'

assert P('///a/b').root == '/'

assert P('//a/b').root == '//'
print("PurePosixPathTest::test_root: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_root: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_root_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_root_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_root_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_root_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_root_common
"""Auto-ported test: PurePosixPathTest::test_root_common (CPython 3.12 oracle)."""


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
sep = self_sep

assert P('').root == ''

assert P('a/b').root == ''

assert P('/').root == sep

assert P('/a/b').root == sep
print("PurePosixPathTest::test_root_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_root_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_stem_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_stem_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_stem_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_stem_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_stem_common
"""Auto-ported test: PurePosixPathTest::test_stem_common (CPython 3.12 oracle)."""


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

assert P('').stem == ''

assert P('.').stem == ''

assert P('..').stem == '..'

assert P('/').stem == ''

assert P('a/b').stem == 'b'

assert P('a/b.py').stem == 'b'

assert P('a/.hgrc').stem == '.hgrc'

assert P('a/.hg.rc').stem == '.hg'

assert P('a/b.tar.gz').stem == 'b.tar'

assert P('a/Some name. Ending with a dot.').stem == 'Some name. Ending with a dot.'
print("PurePosixPathTest::test_stem_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_stem_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_str_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_str_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_str_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_str_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_str_common
"""Auto-ported test: PurePosixPathTest::test_str_common (CPython 3.12 oracle)."""


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
for pathstr in ('a', 'a/b', 'a/b/c', '/', '/a/b', '/a/b/c'):
    _check_str(pathstr, (pathstr,))
_check_str('.', ('',))
print("PurePosixPathTest::test_str_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_str_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_suffix_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_suffix_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_suffix_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_suffix_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_suffix_common
"""Auto-ported test: PurePosixPathTest::test_suffix_common (CPython 3.12 oracle)."""


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

assert P('').suffix == ''

assert P('.').suffix == ''

assert P('..').suffix == ''

assert P('/').suffix == ''

assert P('a/b').suffix == ''

assert P('/a/b').suffix == ''

assert P('/a/b/.').suffix == ''

assert P('a/b.py').suffix == '.py'

assert P('/a/b.py').suffix == '.py'

assert P('a/.hgrc').suffix == ''

assert P('/a/.hgrc').suffix == ''

assert P('a/.hg.rc').suffix == '.rc'

assert P('/a/.hg.rc').suffix == '.rc'

assert P('a/b.tar.gz').suffix == '.gz'

assert P('/a/b.tar.gz').suffix == '.gz'

assert P('a/Some name. Ending with a dot.').suffix == ''

assert P('/a/Some name. Ending with a dot.').suffix == ''
print("PurePosixPathTest::test_suffix_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_suffix_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_suffixes_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_suffixes_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_suffixes_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_suffixes_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_suffixes_common
"""Auto-ported test: PurePosixPathTest::test_suffixes_common (CPython 3.12 oracle)."""


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

assert P('').suffixes == []

assert P('.').suffixes == []

assert P('/').suffixes == []

assert P('a/b').suffixes == []

assert P('/a/b').suffixes == []

assert P('/a/b/.').suffixes == []

assert P('a/b.py').suffixes == ['.py']

assert P('/a/b.py').suffixes == ['.py']

assert P('a/.hgrc').suffixes == []

assert P('/a/.hgrc').suffixes == []

assert P('a/.hg.rc').suffixes == ['.rc']

assert P('/a/.hg.rc').suffixes == ['.rc']

assert P('a/b.tar.gz').suffixes == ['.tar', '.gz']

assert P('/a/b.tar.gz').suffixes == ['.tar', '.gz']

assert P('a/Some name. Ending with a dot.').suffixes == []

assert P('/a/Some name. Ending with a dot.').suffixes == []
print("PurePosixPathTest::test_suffixes_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_suffixes_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_with_name_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_with_name_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_with_name_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_with_name_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_with_name_common
"""Auto-ported test: PurePosixPathTest::test_with_name_common (CPython 3.12 oracle)."""


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

assert P('a/b').with_name('d.xml') == P('a/d.xml')

assert P('/a/b').with_name('d.xml') == P('/a/d.xml')

assert P('a/b.py').with_name('d.xml') == P('a/d.xml')

assert P('/a/b.py').with_name('d.xml') == P('/a/d.xml')

assert P('a/Dot ending.').with_name('d.xml') == P('a/d.xml')

assert P('/a/Dot ending.').with_name('d.xml') == P('/a/d.xml')

try:
    P('').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('.').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('/').with_name('d.xml')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('/c')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('c/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_name('c/d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PurePosixPathTest::test_with_name_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_with_name_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/pure_posix_path_test__test_with_stem_common.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_pure_posix_path_test__test_with_stem_common() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_posix_path_test__test_with_stem_common"
# subject = "cpython.test_pathlib.PurePosixPathTest.test_with_stem_common"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_pathlib.py::PurePosixPathTest::test_with_stem_common
"""Auto-ported test: PurePosixPathTest::test_with_stem_common (CPython 3.12 oracle)."""


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

assert P('a/b').with_stem('d') == P('a/d')

assert P('/a/b').with_stem('d') == P('/a/d')

assert P('a/b.py').with_stem('d') == P('a/d.py')

assert P('/a/b.py').with_stem('d') == P('/a/d.py')

assert P('/a/b.tar.gz').with_stem('d') == P('/a/d.gz')

assert P('a/Dot ending.').with_stem('d') == P('a/d')

assert P('/a/Dot ending.').with_stem('d') == P('/a/d')

try:
    P('').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('.').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('/').with_stem('d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('.')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('/c')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('c/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    P('a/b').with_stem('c/d')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PurePosixPathTest::test_with_stem_common: ok")
"###);
    assert_output(&out, r###"PurePosixPathTest::test_with_stem_common: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/purepath_match_patterns.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_purepath_match_patterns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "purepath_match_patterns"
# subject = "pathlib.PurePosixPath.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath.match: PurePath.match glob semantics: tail-matching of relative patterns, wildcards, leading-slash anchoring, '**' segment spanning, the case_sensitive flag, and empty-path vs '*'/'**' behavior"""
import pathlib

P = pathlib.PurePosixPath

# A relative pattern matches against the tail of the path.
assert P("a/b.py").match("b.py"), "tail match"
assert P("/a/b.py").match("b.py"), "tail match on absolute"
assert not P("a.py").match("b.py"), "name mismatch"
assert not P("b.py/c").match("b.py"), "must match final component"

# Wildcards.
assert P("a/b.py").match("*.py"), "* suffix"
assert not P("b.pyc").match("*.py"), "* must be exact suffix"
assert P("ab/c.py").match("a*/*.py"), "multi-segment wildcard"
assert not P("ab/c.py/d").match("a*/*.py"), "trailing component breaks match"

# A leading slash anchors the pattern to the path root.
assert P("/b.py").match("/*.py"), "anchored match"
assert not P("a/b.py").match("/*.py"), "anchored pattern rejects relative path"
assert P("/a/b.py").match("/a/*.py"), "anchored multi-segment"
assert not P("/a/b/c.py").match("/a/*.py"), "* spans one segment only"

# '**' matches zero-or-more segments only when explicitly anchored.
assert not P("/a/b/c.py").match("/**/*.py"), "leading /** still needs a segment"
assert P("/a/b/c.py").match("/a/**/*.py"), "/** spans segments"

# case_sensitive overrides the flavour default.
assert not P("A.py").match("a.PY", case_sensitive=True), "case-sensitive mismatch"
assert P("A.py").match("a.PY", case_sensitive=False), "case-insensitive match"
assert P("/a/b/c.py").match("/A/*/*.Py", case_sensitive=False), "ci anchored"

# Empty path matches '**' (zero segments) but not '*'.
assert not P().match("*"), "empty path vs *"
assert P().match("**"), "empty path vs **"
assert not P().match("**/*"), "empty path vs **/*"
print("purepath_match_patterns OK")
"###);
    assert_output(&out, r###"purepath_match_patterns OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/relative_to_strips_prefix.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_relative_to_strips_prefix() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "relative_to_strips_prefix"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('/usr/local/bin/python').relative_to('/usr') == Path('local/bin/python')"""
import pathlib

Path = pathlib.Path

_abs = Path("/usr/local/bin/python")
_rel = _abs.relative_to("/usr")
assert _rel == Path("local/bin/python"), f"relative_to = {_rel!r}"
print("relative_to_strips_prefix OK")
"###);
    assert_output(&out, r###"relative_to_strips_prefix OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/resolve_dot_is_cwd.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_resolve_dot_is_cwd() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "resolve_dot_is_cwd"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('.').resolve() returns an absolute path equal to Path.cwd()"""
import pathlib

Path = pathlib.Path

_resolved = Path(".").resolve()
assert _resolved.is_absolute(), f"resolve is absolute: {_resolved!r}"
assert _resolved == Path.cwd(), "resolve('.') == cwd"
print("resolve_dot_is_cwd OK")
"###);
    assert_output(&out, r###"resolve_dot_is_cwd OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/serialization_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_serialization_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "serialization_roundtrip"
# subject = "pathlib.PurePath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath: Path serialization: pickle round-trips preserve class/value/hash/str across protocols, bytes()/os.fspath give the OS-encoded string, as_posix always uses forward slashes, as_uri percent-encodes absolute POSIX paths, and equality against foreign types is False without raising"""
import pathlib

import os
import pickle
PurePath = pathlib.PurePath
PurePosixPath = pathlib.PurePosixPath
PosixPath = pathlib.PosixPath

# Pickle round-trips preserve class, value, hash, and string form.
_p = PurePath("/a/b")
for _proto in range(0, pickle.HIGHEST_PROTOCOL + 1):
    _back = pickle.loads(pickle.dumps(_p, _proto))
    assert _back.__class__ is _p.__class__, f"class @ proto {_proto}"
    assert _back == _p, f"value @ proto {_proto}"
    assert hash(_back) == hash(_p), f"hash @ proto {_proto}"
    assert str(_back) == str(_p), f"str @ proto {_proto}"

# bytes() and os.fspath give the OS-encoded path string.
_sep = os.fsencode(os.sep)
assert bytes(PurePath("a/b")) == b"a" + _sep + b"b", "bytes() encoding"
assert os.fspath(PurePath("a/b")) == os.path.join("a", "b"), "os.fspath"

# as_posix always uses forward slashes.
for _s in ("a", "a/b", "/", "/a/b/c"):
    assert PurePath(_s).as_posix() == _s, f"as_posix {_s!r}"

# Absolute POSIX paths produce file:// URIs, percent-encoding special bytes.
assert PosixPath("/").as_uri() == "file:///", "root uri"
assert PosixPath("/a/b.c").as_uri() == "file:///a/b.c", "plain uri"
assert PosixPath("/a/b%#c").as_uri() == "file:///a/b%25%23c", "encoded uri"

# Equality against unrelated types is always False (never raises).
assert PurePath() != "", "path != str"
assert PurePath() != {}, "path != dict"
assert PurePath() != int, "path != type"
assert PurePath("a/b") == PurePath("a", "b"), "value equality across constructors"
print("serialization_roundtrip OK")
"###);
    assert_output(&out, r###"serialization_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/slash_join_returns_path.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_slash_join_returns_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "slash_join_returns_path"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: the / operator joins components producing a new Path: Path('/usr')/'local'/'bin' == Path('/usr/local/bin') and the result is a Path instance"""
import pathlib

Path = pathlib.Path

_base = Path("/usr")
_full = _base / "local" / "bin"
assert _full == Path("/usr/local/bin"), f"join = {_full!r}"
assert isinstance(_full, Path), "join returns Path"
print("slash_join_returns_path OK")
"###);
    assert_output(&out, r###"slash_join_returns_path OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/sort_orders_by_string_value.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_sort_orders_by_string_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "sort_orders_by_string_value"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: sorting a list of Paths orders them by their string value: sorted([Path('/b'),Path('/a'),Path('/c')]) == [Path('/a'),Path('/b'),Path('/c')]"""
import pathlib

Path = pathlib.Path

_paths = [Path("/b"), Path("/a"), Path("/c")]
_sorted = sorted(_paths)
assert _sorted == [Path("/a"), Path("/b"), Path("/c")], f"sorted paths = {_sorted!r}"
print("sort_orders_by_string_value OK")
"###);
    assert_output(&out, r###"sort_orders_by_string_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/stat_returns_stat_result.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_stat_returns_stat_result() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "stat_returns_stat_result"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('.').stat() returns a stat result exposing st_size and st_mtime attributes"""
import pathlib

Path = pathlib.Path

_stat = Path(".").stat()
assert hasattr(_stat, "st_size"), "stat has st_size"
assert hasattr(_stat, "st_mtime"), "stat has st_mtime"
print("stat_returns_stat_result OK")
"###);
    assert_output(&out, r###"stat_returns_stat_result OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/suffixes_and_multi_suffix_stem.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_suffixes_and_multi_suffix_stem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "suffixes_and_multi_suffix_stem"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('file.tar.gz').suffixes == ['.tar', '.gz'] and its stem keeps all but the last suffix ('file.tar')"""
import pathlib

Path = pathlib.Path

_multi = Path("file.tar.gz")
assert _multi.suffixes == [".tar", ".gz"], f"suffixes = {_multi.suffixes!r}"
assert _multi.stem == "file.tar", f"multi stem = {_multi.stem!r}"
print("suffixes_and_multi_suffix_stem OK")
"###);
    assert_output(&out, r###"suffixes_and_multi_suffix_stem OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/with_suffix_name_stem.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_with_suffix_name_stem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "with_suffix_name_stem"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: with_suffix/.with_name/.with_stem return new paths: file.txt.with_suffix('.csv')==file.csv, /tmp/old.txt.with_name('new.csv')==/tmp/new.csv, /a/b.tar.gz.with_stem('d')==/a/d.gz"""
import pathlib

Path = pathlib.Path

_new_ext = Path("file.txt").with_suffix(".csv")
assert _new_ext == Path("file.csv"), f"with_suffix = {_new_ext!r}"
_new_name = Path("/tmp/old.txt").with_name("new.csv")
assert _new_name == Path("/tmp/new.csv"), f"with_name = {_new_name!r}"
_new_stem = Path("/a/b.tar.gz").with_stem("d")
assert _new_stem == Path("/a/d.gz"), f"with_stem = {_new_stem!r}"
print("with_suffix_name_stem OK")
"###);
    assert_output(&out, r###"with_suffix_name_stem OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pathlib/write_read_text_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pathlib_write_read_text_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "write_read_text_roundtrip"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory, Path.write_text/read_text round-trip the same string and the file then reports exists()/is_file()"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _f = Path(_tmpdir) / "test.txt"
    _f.write_text("hello world")
    _content = _f.read_text()
    assert _content == "hello world", f"read_text = {_content!r}"
    assert _f.exists(), "file exists"
    assert _f.is_file(), "is file"
print("write_read_text_roundtrip OK")
"###);
    assert_output(&out, r###"write_read_text_roundtrip OK
"###);
}
