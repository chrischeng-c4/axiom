use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/filecmp/dir_compare_test_case__test_cmpfiles.py`.
#[test]
fn test_gen_behavior_std_libs_filecmp_dir_compare_test_case__test_cmpfiles() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "dir_compare_test_case__test_cmpfiles"
# subject = "cpython.test_filecmp.DirCompareTestCase.test_cmpfiles"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_filecmp.py::DirCompareTestCase::test_cmpfiles
"""Auto-ported test: DirCompareTestCase::test_cmpfiles (CPython 3.12 oracle)."""


import filecmp
import os
import shutil
import tempfile
import unittest
from test import support
from test.support import os_helper


# --- test body ---
tmpdir = tempfile.gettempdir()
self_dir = os.path.join(tmpdir, 'dir')
self_dir_same = os.path.join(tmpdir, 'dir-same')
self_dir_diff = os.path.join(tmpdir, 'dir-diff')
self_dir_ignored = os.path.join(self_dir_same, '.hg')
self_caseinsensitive = os.path.normcase('A') == os.path.normcase('a')
data = 'Contents of file go here.\n'
for dir in (self_dir, self_dir_same, self_dir_diff, self_dir_ignored):
    shutil.rmtree(dir, True)
    os.mkdir(dir)
    subdir_path = os.path.join(dir, 'subdir')
    os.mkdir(subdir_path)
    if self_caseinsensitive and dir is self_dir_same:
        fn = 'FiLe'
    else:
        fn = 'file'
    with open(os.path.join(dir, fn), 'w', encoding='utf-8') as output:
        output.write(data)
with open(os.path.join(self_dir_diff, 'file2'), 'w', encoding='utf-8') as output:
    output.write('An extra file.\n')

assert filecmp.cmpfiles(self_dir, self_dir, ['file']) == (['file'], [], [])

assert filecmp.cmpfiles(self_dir, self_dir_same, ['file']) == (['file'], [], [])

assert filecmp.cmpfiles(self_dir, self_dir, ['file'], shallow=False) == (['file'], [], [])

assert filecmp.cmpfiles(self_dir, self_dir_same, ['file'], shallow=False)
with open(os.path.join(self_dir, 'file2'), 'w', encoding='utf-8') as output:
    output.write('Different contents.\n')

assert not filecmp.cmpfiles(self_dir, self_dir_same, ['file', 'file2']) == (['file'], ['file2'], [])
print("DirCompareTestCase::test_cmpfiles: ok")
"###);
    assert_output(&out, r###"DirCompareTestCase::test_cmpfiles: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/filecmp/dir_compare_test_case__test_default_ignores.py`.
#[test]
fn test_gen_behavior_std_libs_filecmp_dir_compare_test_case__test_default_ignores() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "dir_compare_test_case__test_default_ignores"
# subject = "cpython.test_filecmp.DirCompareTestCase.test_default_ignores"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_filecmp.py::DirCompareTestCase::test_default_ignores
"""Auto-ported test: DirCompareTestCase::test_default_ignores (CPython 3.12 oracle)."""


import filecmp
import os
import shutil
import tempfile
import unittest
from test import support
from test.support import os_helper


# --- test body ---
tmpdir = tempfile.gettempdir()
self_dir = os.path.join(tmpdir, 'dir')
self_dir_same = os.path.join(tmpdir, 'dir-same')
self_dir_diff = os.path.join(tmpdir, 'dir-diff')
self_dir_ignored = os.path.join(self_dir_same, '.hg')
self_caseinsensitive = os.path.normcase('A') == os.path.normcase('a')
data = 'Contents of file go here.\n'
for dir in (self_dir, self_dir_same, self_dir_diff, self_dir_ignored):
    shutil.rmtree(dir, True)
    os.mkdir(dir)
    subdir_path = os.path.join(dir, 'subdir')
    os.mkdir(subdir_path)
    if self_caseinsensitive and dir is self_dir_same:
        fn = 'FiLe'
    else:
        fn = 'file'
    with open(os.path.join(dir, fn), 'w', encoding='utf-8') as output:
        output.write(data)
with open(os.path.join(self_dir_diff, 'file2'), 'w', encoding='utf-8') as output:
    output.write('An extra file.\n')

assert '.hg' in filecmp.DEFAULT_IGNORES
print("DirCompareTestCase::test_default_ignores: ok")
"###);
    assert_output(&out, r###"DirCompareTestCase::test_default_ignores: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/filecmp/file_compare_test_case__test_cache_clear.py`.
#[test]
fn test_gen_behavior_std_libs_filecmp_file_compare_test_case__test_cache_clear() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "file_compare_test_case__test_cache_clear"
# subject = "cpython.test_filecmp.FileCompareTestCase.test_cache_clear"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_filecmp.py::FileCompareTestCase::test_cache_clear
"""Auto-ported test: FileCompareTestCase::test_cache_clear (CPython 3.12 oracle)."""


import filecmp
import os
import shutil
import tempfile
import unittest
from test import support
from test.support import os_helper


# --- test body ---
self_name = os_helper.TESTFN
self_name_same = os_helper.TESTFN + '-same'
self_name_diff = os_helper.TESTFN + '-diff'
data = 'Contents of file go here.\n'
for name in [self_name, self_name_same, self_name_diff]:
    with open(name, 'w', encoding='utf-8') as output:
        output.write(data)
with open(self_name_diff, 'a+', encoding='utf-8') as output:
    output.write('An extra line.\n')
self_dir = tempfile.gettempdir()
first_compare = filecmp.cmp(self_name, self_name_same, shallow=False)
second_compare = filecmp.cmp(self_name, self_name_diff, shallow=False)
filecmp.clear_cache()

assert len(filecmp._cache) == 0
print("FileCompareTestCase::test_cache_clear: ok")
"###);
    assert_output(&out, r###"FileCompareTestCase::test_cache_clear: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/filecmp/file_compare_test_case__test_different.py`.
#[test]
fn test_gen_behavior_std_libs_filecmp_file_compare_test_case__test_different() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "file_compare_test_case__test_different"
# subject = "cpython.test_filecmp.FileCompareTestCase.test_different"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_filecmp.py::FileCompareTestCase::test_different
"""Auto-ported test: FileCompareTestCase::test_different (CPython 3.12 oracle)."""


import filecmp
import os
import shutil
import tempfile
import unittest
from test import support
from test.support import os_helper


# --- test body ---
self_name = os_helper.TESTFN
self_name_same = os_helper.TESTFN + '-same'
self_name_diff = os_helper.TESTFN + '-diff'
data = 'Contents of file go here.\n'
for name in [self_name, self_name_same, self_name_diff]:
    with open(name, 'w', encoding='utf-8') as output:
        output.write(data)
with open(self_name_diff, 'a+', encoding='utf-8') as output:
    output.write('An extra line.\n')
self_dir = tempfile.gettempdir()

assert not filecmp.cmp(self_name, self_name_diff)

assert not filecmp.cmp(self_name, self_dir)
print("FileCompareTestCase::test_different: ok")
"###);
    assert_output(&out, r###"FileCompareTestCase::test_different: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/filecmp/file_compare_test_case__test_matching.py`.
#[test]
fn test_gen_behavior_std_libs_filecmp_file_compare_test_case__test_matching() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "file_compare_test_case__test_matching"
# subject = "cpython.test_filecmp.FileCompareTestCase.test_matching"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_filecmp.py::FileCompareTestCase::test_matching
"""Auto-ported test: FileCompareTestCase::test_matching (CPython 3.12 oracle)."""


import filecmp
import os
import shutil
import tempfile
import unittest
from test import support
from test.support import os_helper


# --- test body ---
self_name = os_helper.TESTFN
self_name_same = os_helper.TESTFN + '-same'
self_name_diff = os_helper.TESTFN + '-diff'
data = 'Contents of file go here.\n'
for name in [self_name, self_name_same, self_name_diff]:
    with open(name, 'w', encoding='utf-8') as output:
        output.write(data)
with open(self_name_diff, 'a+', encoding='utf-8') as output:
    output.write('An extra line.\n')
self_dir = tempfile.gettempdir()

assert filecmp.cmp(self_name, self_name)

assert filecmp.cmp(self_name, self_name, shallow=False)

assert filecmp.cmp(self_name, self_name_same)

assert filecmp.cmp(self_name, self_name_same, shallow=False)
print("FileCompareTestCase::test_matching: ok")
"###);
    assert_output(&out, r###"FileCompareTestCase::test_matching: ok
"###);
}
