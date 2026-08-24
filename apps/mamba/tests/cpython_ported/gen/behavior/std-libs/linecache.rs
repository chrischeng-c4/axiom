use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/linecache/checkcache_drops_vanished_file.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_checkcache_drops_vanished_file() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "checkcache_drops_vanished_file"
# subject = "linecache.checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.checkcache: checkcache drops the cache entry once its backing file is deleted, over a TemporaryDirectory"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    gone = os.path.join(d, "gone.py")
    with open(gone, "w") as fh:
        fh.write("x = 1\ny = 2\n")
    linecache.getline(gone, 1)
    assert gone in linecache.cache, "gone cached"
    os.unlink(gone)
    linecache.checkcache(gone)
    assert gone not in linecache.cache, "vanished dropped"
print("checkcache_drops_vanished_file OK")
"###);
    assert_output(&out, r###"checkcache_drops_vanished_file OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/checkcache_keeps_unchanged_file.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_checkcache_keeps_unchanged_file() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "checkcache_keeps_unchanged_file"
# subject = "linecache.checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.checkcache: checkcache keeps the cache entry of an unchanged on-disk file, over a TemporaryDirectory"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    keep = os.path.join(d, "keep.py")
    with open(keep, "w") as fh:
        fh.write("x = 1\ny = 2\n")
    linecache.getline(keep, 1)
    assert keep in linecache.cache, "keep cached"
    linecache.checkcache(keep)
    assert keep in linecache.cache, "unchanged kept"
print("checkcache_keeps_unchanged_file OK")
"###);
    assert_output(&out, r###"checkcache_keeps_unchanged_file OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/clearcache_empties_cache.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_clearcache_empties_cache() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "clearcache_empties_cache"
# subject = "linecache.clearcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.clearcache: clearcache removes every populated entry from linecache.cache, over a TemporaryDirectory"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    paths = []
    for name in ("one", "two"):
        p = os.path.join(d, name + ".py")
        with open(p, "w") as fh:
            fh.write("x = 1\ny = 2\n")
        paths.append(p)
        linecache.getline(p, 1)  # populate the cache
    assert all(p in linecache.cache for p in paths), "populated"
    linecache.clearcache()
    assert all(p not in linecache.cache for p in paths), "cleared"
print("clearcache_empties_cache OK")
"###);
    assert_output(&out, r###"clearcache_empties_cache OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/getline_bad_filename_returns_empty.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_getline_bad_filename_returns_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_bad_filename_returns_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline with an empty or syntactically-invalid filename returns '' without raising"""
import linecache

linecache.clearcache()
assert linecache.getline("", 1) == "", "getline empty name"
assert linecache.getline("!@$)(!@#_1", 1) == "", "getline invalid name"
print("getline_bad_filename_returns_empty OK")
"###);
    assert_output(&out, r###"getline_bad_filename_returns_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/getline_one_based_indexing.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_getline_one_based_indexing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_one_based_indexing"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline is 1-based: line 1 is the first source line, line 3 the third, over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    assert linecache.getline(fn, 1).rstrip() == "alpha", "getline(1)"
    assert linecache.getline(fn, 3).rstrip() == "charlie", "getline(3)"
finally:
    os.unlink(fn)
print("getline_one_based_indexing OK")
"###);
    assert_output(&out, r###"getline_one_based_indexing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/getline_populates_cache.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_getline_populates_cache() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_populates_cache"
# subject = "linecache.cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.cache: getline populates linecache.cache, keyed by filename, for each file read, over a TemporaryDirectory"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    paths = []
    for name in ("one", "two"):
        p = os.path.join(d, name + ".py")
        with open(p, "w") as fh:
            fh.write("x = 1\ny = 2\n")
        paths.append(p)
        linecache.getline(p, 1)  # populate the cache
    assert all(p in linecache.cache for p in paths), "populated"
print("getline_populates_cache OK")
"###);
    assert_output(&out, r###"getline_populates_cache OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/getline_round_trips_with_open.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_getline_round_trips_with_open() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_round_trips_with_open"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: every line read directly via open() equals getline of that 1-based lineno, over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    with open(fn, encoding="utf-8") as f:
        for index, line in enumerate(f):
            assert line == linecache.getline(fn, index + 1), "round-trip"
finally:
    os.unlink(fn)
print("getline_round_trips_with_open OK")
"###);
    assert_output(&out, r###"getline_round_trips_with_open OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/getline_zero_and_negative_return_empty.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_getline_zero_and_negative_return_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_zero_and_negative_return_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline with lineno 0, -1, or 2**15 returns '' (only 1..len map to lines), over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    assert linecache.getline(fn, 99) == "", "getline(99)"
    assert linecache.getline(fn, 2 ** 15) == "", "getline(2**15)"
    assert linecache.getline(fn, 0) == "", "getline(0)"
    assert linecache.getline(fn, -1) == "", "getline(-1)"
finally:
    os.unlink(fn)
print("getline_zero_and_negative_return_empty OK")
"###);
    assert_output(&out, r###"getline_zero_and_negative_return_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/getlines_returns_list_per_line.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_getlines_returns_list_per_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getlines_returns_list_per_line"
# subject = "linecache.getlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getlines: getlines returns a list[str] with one newline-terminated entry per source line, over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    lines = linecache.getlines(fn)
    assert type(lines) is list, "getlines is list"
    assert len(lines) == 4, "getlines len"
    assert lines[0].rstrip() == "alpha", "getlines[0]"
    assert lines[3] == "delta\n", "getlines[3] newline-terminated"
finally:
    os.unlink(fn)
print("getlines_returns_list_per_line OK")
"###);
    assert_output(&out, r###"getlines_returns_list_per_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/lazycache_already_cached_returns_false.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_lazycache_already_cached_returns_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_already_cached_returns_false"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache leaves a fully-cached 4-tuple entry untouched and returns False"""
import linecache

SRC = "line one\nline two\nline three\n"
FAKE = "/no/such/dir/lazy_module.py"  # cacheable name, file never exists


class Loader:
    def get_source(self, name):
        return SRC


def fresh_globals():
    return {"__name__": "lazy.mod", "__loader__": Loader()}


linecache.clearcache()
before = linecache.getlines(FAKE, fresh_globals())
assert before == ["line one\n", "line two\n", "line three\n"], "pre-cache"
assert linecache.lazycache(FAKE, fresh_globals()) is False, "already cached -> False"
assert len(linecache.cache[FAKE]) == 4, "stays full 4-tuple"
print("lazycache_already_cached_returns_false OK")
"###);
    assert_output(&out, r###"lazycache_already_cached_returns_false OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/lazycache_bad_filename_returns_false.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_lazycache_bad_filename_returns_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_bad_filename_returns_false"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache rejects non-cacheable names (empty string, angle-bracketed) and returns False"""
import linecache

SRC = "line one\nline two\nline three\n"


class Loader:
    def get_source(self, name):
        return SRC


def fresh_globals():
    return {"__name__": "lazy.mod", "__loader__": Loader()}


linecache.clearcache()
assert linecache.lazycache("", fresh_globals()) is False, "empty name"
assert linecache.lazycache("<foo>", fresh_globals()) is False, "<bracket> name"
print("lazycache_bad_filename_returns_false OK")
"###);
    assert_output(&out, r###"lazycache_bad_filename_returns_false OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/lazycache_no_globals_returns_false.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_lazycache_no_globals_returns_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_no_globals_returns_false"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache with module_globals=None has nothing to load from, returns False, and caches nothing"""
import linecache

FAKE = "/no/such/dir/lazy_module.py"  # cacheable name, file never exists

linecache.clearcache()
assert linecache.lazycache(FAKE, None) is False, "no globals -> False"
assert FAKE not in linecache.cache, "no globals -> uncached"
print("lazycache_no_globals_returns_false OK")
"###);
    assert_output(&out, r###"lazycache_no_globals_returns_false OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/lazycache_registers_then_materializes.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_lazycache_registers_then_materializes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "lazycache_registers_then_materializes"
# subject = "linecache.lazycache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.lazycache: lazycache registers a lazy 1-tuple entry for an absent file via a loader's get_source and returns True; a later getlines materializes the source"""
import linecache

SRC = "line one\nline two\nline three\n"
FAKE = "/no/such/dir/lazy_module.py"  # cacheable name, file never exists


class Loader:
    def get_source(self, name):
        return SRC


def fresh_globals():
    return {"__name__": "lazy.mod", "__loader__": Loader()}


linecache.clearcache()
assert linecache.lazycache(FAKE, fresh_globals()) is True, "lazy registered"
assert len(linecache.cache[FAKE]) == 1, "entry is lazy 1-tuple"
# getlines materializes the source via the loader.
assert linecache.getlines(FAKE) == ["line one\n", "line two\n", "line three\n"], "materialized"
print("lazycache_registers_then_materializes OK")
"###);
    assert_output(&out, r###"lazycache_registers_then_materializes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/line_cache_tests__test_checkcache.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_line_cache_tests__test_checkcache() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_checkcache"
# subject = "cpython.test_linecache.LineCacheTests.test_checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_checkcache
"""Auto-ported test: LineCacheTests::test_checkcache (CPython 3.12 oracle)."""


import linecache
import unittest
import os.path
import tempfile
import tokenize
from importlib.machinery import ModuleSpec
from test import support
from test.support import os_helper


' Tests for the linecache module '

FILENAME = linecache.__file__

NONEXISTENT_FILENAME = FILENAME + '.missing'

INVALID_NAME = '!@$)(!@#_1'

EMPTY = ''

TEST_PATH = os.path.dirname(__file__)

MODULES = 'linecache abc'.split()

MODULE_PATH = os.path.dirname(FILENAME)

SOURCE_1 = '\n" Docstring "\n\ndef function():\n    return result\n\n'

SOURCE_2 = '\ndef f():\n    return 1 + 1\n\na = f()\n\n'

SOURCE_3 = '\ndef f():\n    return 3'

class TempFile:

    def setUp(self):
        super().setUp()
        with tempfile.NamedTemporaryFile(delete=False) as fp:
            self.file_name = fp.name
            fp.write(self.file_byte_string)
        self.addCleanup(os_helper.unlink, self.file_name)

class FakeLoader:

    def get_source(self, fullname):
        return f'source for {fullname}'

class NoSourceLoader:

    def get_source(self, fullname):
        return None


# --- test body ---
getline = linecache.getline
source_name = os_helper.TESTFN + '.py'
pass
with open(source_name, 'w', encoding='utf-8') as source:
    source.write(SOURCE_1)
getline(source_name, 1)
source_list = []
with open(source_name, encoding='utf-8') as source:
    for index, line in enumerate(source):

        assert line == getline(source_name, index + 1)
        source_list.append(line)
with open(source_name, 'w', encoding='utf-8') as source:
    source.write(SOURCE_2)
linecache.checkcache('dummy')
for index, line in enumerate(source_list):

    assert line == getline(source_name, index + 1)
linecache.checkcache(source_name)
with open(source_name, encoding='utf-8') as source:
    for index, line in enumerate(source):

        assert line == getline(source_name, index + 1)
        source_list.append(line)
print("LineCacheTests::test_checkcache: ok")
"###);
    assert_output(&out, r###"LineCacheTests::test_checkcache: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/line_cache_tests__test_lazycache_bad_filename.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_line_cache_tests__test_lazycache_bad_filename() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_lazycache_bad_filename"
# subject = "cpython.test_linecache.LineCacheTests.test_lazycache_bad_filename"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_lazycache_bad_filename
"""Auto-ported test: LineCacheTests::test_lazycache_bad_filename (CPython 3.12 oracle)."""


import linecache
import unittest
import os.path
import tempfile
import tokenize
from importlib.machinery import ModuleSpec
from test import support
from test.support import os_helper


' Tests for the linecache module '

FILENAME = linecache.__file__

NONEXISTENT_FILENAME = FILENAME + '.missing'

INVALID_NAME = '!@$)(!@#_1'

EMPTY = ''

TEST_PATH = os.path.dirname(__file__)

MODULES = 'linecache abc'.split()

MODULE_PATH = os.path.dirname(FILENAME)

SOURCE_1 = '\n" Docstring "\n\ndef function():\n    return result\n\n'

SOURCE_2 = '\ndef f():\n    return 1 + 1\n\na = f()\n\n'

SOURCE_3 = '\ndef f():\n    return 3'

class TempFile:

    def setUp(self):
        super().setUp()
        with tempfile.NamedTemporaryFile(delete=False) as fp:
            self.file_name = fp.name
            fp.write(self.file_byte_string)
        self.addCleanup(os_helper.unlink, self.file_name)

class FakeLoader:

    def get_source(self, fullname):
        return f'source for {fullname}'

class NoSourceLoader:

    def get_source(self, fullname):
        return None


# --- test body ---
linecache.clearcache()

assert False == linecache.lazycache('', globals())

assert False == linecache.lazycache('<foo>', globals())
print("LineCacheTests::test_lazycache_bad_filename: ok")
"###);
    assert_output(&out, r###"LineCacheTests::test_lazycache_bad_filename: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/line_cache_tests__test_lazycache_check.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_line_cache_tests__test_lazycache_check() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_lazycache_check"
# subject = "cpython.test_linecache.LineCacheTests.test_lazycache_check"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_lazycache_check
"""Auto-ported test: LineCacheTests::test_lazycache_check (CPython 3.12 oracle)."""


import linecache
import unittest
import os.path
import tempfile
import tokenize
from importlib.machinery import ModuleSpec
from test import support
from test.support import os_helper


' Tests for the linecache module '

FILENAME = linecache.__file__

NONEXISTENT_FILENAME = FILENAME + '.missing'

INVALID_NAME = '!@$)(!@#_1'

EMPTY = ''

TEST_PATH = os.path.dirname(__file__)

MODULES = 'linecache abc'.split()

MODULE_PATH = os.path.dirname(FILENAME)

SOURCE_1 = '\n" Docstring "\n\ndef function():\n    return result\n\n'

SOURCE_2 = '\ndef f():\n    return 1 + 1\n\na = f()\n\n'

SOURCE_3 = '\ndef f():\n    return 3'

class TempFile:

    def setUp(self):
        super().setUp()
        with tempfile.NamedTemporaryFile(delete=False) as fp:
            self.file_name = fp.name
            fp.write(self.file_byte_string)
        self.addCleanup(os_helper.unlink, self.file_name)

class FakeLoader:

    def get_source(self, fullname):
        return f'source for {fullname}'

class NoSourceLoader:

    def get_source(self, fullname):
        return None


# --- test body ---
linecache.clearcache()
linecache.lazycache(NONEXISTENT_FILENAME, globals())
linecache.checkcache()
print("LineCacheTests::test_lazycache_check: ok")
"###);
    assert_output(&out, r###"LineCacheTests::test_lazycache_check: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/line_cache_tests__test_lazycache_no_globals.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_line_cache_tests__test_lazycache_no_globals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_lazycache_no_globals"
# subject = "cpython.test_linecache.LineCacheTests.test_lazycache_no_globals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_lazycache_no_globals
"""Auto-ported test: LineCacheTests::test_lazycache_no_globals (CPython 3.12 oracle)."""


import linecache
import unittest
import os.path
import tempfile
import tokenize
from importlib.machinery import ModuleSpec
from test import support
from test.support import os_helper


' Tests for the linecache module '

FILENAME = linecache.__file__

NONEXISTENT_FILENAME = FILENAME + '.missing'

INVALID_NAME = '!@$)(!@#_1'

EMPTY = ''

TEST_PATH = os.path.dirname(__file__)

MODULES = 'linecache abc'.split()

MODULE_PATH = os.path.dirname(FILENAME)

SOURCE_1 = '\n" Docstring "\n\ndef function():\n    return result\n\n'

SOURCE_2 = '\ndef f():\n    return 1 + 1\n\na = f()\n\n'

SOURCE_3 = '\ndef f():\n    return 3'

class TempFile:

    def setUp(self):
        super().setUp()
        with tempfile.NamedTemporaryFile(delete=False) as fp:
            self.file_name = fp.name
            fp.write(self.file_byte_string)
        self.addCleanup(os_helper.unlink, self.file_name)

class FakeLoader:

    def get_source(self, fullname):
        return f'source for {fullname}'

class NoSourceLoader:

    def get_source(self, fullname):
        return None


# --- test body ---
lines = linecache.getlines(FILENAME)
linecache.clearcache()

assert False == linecache.lazycache(FILENAME, None)

assert lines == linecache.getlines(FILENAME)
print("LineCacheTests::test_lazycache_no_globals: ok")
"###);
    assert_output(&out, r###"LineCacheTests::test_lazycache_no_globals: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/line_cache_tests__test_lazycache_provide_after_failed_lookup.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_line_cache_tests__test_lazycache_provide_after_failed_lookup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_lazycache_provide_after_failed_lookup"
# subject = "cpython.test_linecache.LineCacheTests.test_lazycache_provide_after_failed_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_lazycache_provide_after_failed_lookup
"""Auto-ported test: LineCacheTests::test_lazycache_provide_after_failed_lookup (CPython 3.12 oracle)."""


import linecache
import unittest
import os.path
import tempfile
import tokenize
from importlib.machinery import ModuleSpec
from test import support
from test.support import os_helper


' Tests for the linecache module '

FILENAME = linecache.__file__

NONEXISTENT_FILENAME = FILENAME + '.missing'

INVALID_NAME = '!@$)(!@#_1'

EMPTY = ''

TEST_PATH = os.path.dirname(__file__)

MODULES = 'linecache abc'.split()

MODULE_PATH = os.path.dirname(FILENAME)

SOURCE_1 = '\n" Docstring "\n\ndef function():\n    return result\n\n'

SOURCE_2 = '\ndef f():\n    return 1 + 1\n\na = f()\n\n'

SOURCE_3 = '\ndef f():\n    return 3'

class TempFile:

    def setUp(self):
        super().setUp()
        with tempfile.NamedTemporaryFile(delete=False) as fp:
            self.file_name = fp.name
            fp.write(self.file_byte_string)
        self.addCleanup(os_helper.unlink, self.file_name)

class FakeLoader:

    def get_source(self, fullname):
        return f'source for {fullname}'

class NoSourceLoader:

    def get_source(self, fullname):
        return None


# --- test body ---
linecache.clearcache()
lines = linecache.getlines(NONEXISTENT_FILENAME, globals())
linecache.clearcache()
linecache.getlines(NONEXISTENT_FILENAME)
linecache.lazycache(NONEXISTENT_FILENAME, globals())

assert lines == linecache.updatecache(NONEXISTENT_FILENAME)
print("LineCacheTests::test_lazycache_provide_after_failed_lookup: ok")
"###);
    assert_output(&out, r###"LineCacheTests::test_lazycache_provide_after_failed_lookup: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/linecache/line_cache_tests__test_no_ending_newline.py`.
#[test]
fn test_gen_behavior_std_libs_linecache_line_cache_tests__test_no_ending_newline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "line_cache_tests__test_no_ending_newline"
# subject = "cpython.test_linecache.LineCacheTests.test_no_ending_newline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_linecache.py::LineCacheTests::test_no_ending_newline
"""Auto-ported test: LineCacheTests::test_no_ending_newline (CPython 3.12 oracle)."""


import linecache
import unittest
import os.path
import tempfile
import tokenize
from importlib.machinery import ModuleSpec
from test import support
from test.support import os_helper


' Tests for the linecache module '

FILENAME = linecache.__file__

NONEXISTENT_FILENAME = FILENAME + '.missing'

INVALID_NAME = '!@$)(!@#_1'

EMPTY = ''

TEST_PATH = os.path.dirname(__file__)

MODULES = 'linecache abc'.split()

MODULE_PATH = os.path.dirname(FILENAME)

SOURCE_1 = '\n" Docstring "\n\ndef function():\n    return result\n\n'

SOURCE_2 = '\ndef f():\n    return 1 + 1\n\na = f()\n\n'

SOURCE_3 = '\ndef f():\n    return 3'

class TempFile:

    def setUp(self):
        super().setUp()
        with tempfile.NamedTemporaryFile(delete=False) as fp:
            self.file_name = fp.name
            fp.write(self.file_byte_string)
        self.addCleanup(os_helper.unlink, self.file_name)

class FakeLoader:

    def get_source(self, fullname):
        return f'source for {fullname}'

class NoSourceLoader:

    def get_source(self, fullname):
        return None


# --- test body ---
pass
with open(os_helper.TESTFN, 'w', encoding='utf-8') as fp:
    fp.write(SOURCE_3)
lines = linecache.getlines(os_helper.TESTFN)

assert lines == ['\n', 'def f():\n', '    return 3\n']
print("LineCacheTests::test_no_ending_newline: ok")
"###);
    assert_output(&out, r###"LineCacheTests::test_no_ending_newline: ok
"###);
}
