# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "dir_compare_test_case__test_dircmp"
# subject = "cpython.test_filecmp.DirCompareTestCase.test_dircmp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_filecmp.py::DirCompareTestCase::test_dircmp
"""Auto-ported test: DirCompareTestCase::test_dircmp (CPython 3.12 oracle)."""


import filecmp
import os
import shutil
import tempfile
import unittest
from test import support
from test.support import os_helper


# --- test body ---
def _assert_lists(actual, expected):
    """Assert that two lists are equal, up to ordering."""

    assert sorted(actual) == sorted(expected)

def _assert_report(dircmp_report, expected_report_lines):
    with support.captured_stdout() as stdout:
        dircmp_report()
        report_lines = stdout.getvalue().strip().split('\n')

        assert report_lines == expected_report_lines
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
left_dir, right_dir = (self_dir, self_dir_same)
d = filecmp.dircmp(left_dir, right_dir)

assert d.left == left_dir

assert d.right == right_dir
if self_caseinsensitive:
    _assert_lists(d.left_list, ['file', 'subdir'])
    _assert_lists(d.right_list, ['FiLe', 'subdir'])
else:
    _assert_lists(d.left_list, ['file', 'subdir'])
    _assert_lists(d.right_list, ['file', 'subdir'])
_assert_lists(d.common, ['file', 'subdir'])
_assert_lists(d.common_dirs, ['subdir'])

assert d.left_only == []

assert d.right_only == []

assert d.same_files == ['file']

assert d.diff_files == []
expected_report = ['diff {} {}'.format(self_dir, self_dir_same), "Identical files : ['file']", "Common subdirectories : ['subdir']"]
_assert_report(d.report, expected_report)
left_dir, right_dir = (self_dir, self_dir_diff)
d = filecmp.dircmp(left_dir, right_dir)

assert d.left == left_dir

assert d.right == right_dir
_assert_lists(d.left_list, ['file', 'subdir'])
_assert_lists(d.right_list, ['file', 'file2', 'subdir'])
_assert_lists(d.common, ['file', 'subdir'])
_assert_lists(d.common_dirs, ['subdir'])

assert d.left_only == []

assert d.right_only == ['file2']

assert d.same_files == ['file']

assert d.diff_files == []
expected_report = ['diff {} {}'.format(self_dir, self_dir_diff), "Only in {} : ['file2']".format(self_dir_diff), "Identical files : ['file']", "Common subdirectories : ['subdir']"]
_assert_report(d.report, expected_report)
left_dir, right_dir = (self_dir, self_dir_diff)
shutil.move(os.path.join(self_dir_diff, 'file2'), os.path.join(self_dir, 'file2'))
d = filecmp.dircmp(left_dir, right_dir)

assert d.left == left_dir

assert d.right == right_dir
_assert_lists(d.left_list, ['file', 'file2', 'subdir'])
_assert_lists(d.right_list, ['file', 'subdir'])
_assert_lists(d.common, ['file', 'subdir'])

assert d.left_only == ['file2']

assert d.right_only == []

assert d.same_files == ['file']

assert d.diff_files == []
expected_report = ['diff {} {}'.format(self_dir, self_dir_diff), "Only in {} : ['file2']".format(self_dir), "Identical files : ['file']", "Common subdirectories : ['subdir']"]
_assert_report(d.report, expected_report)
with open(os.path.join(self_dir_diff, 'file2'), 'w', encoding='utf-8') as output:
    output.write('Different contents.\n')
d = filecmp.dircmp(self_dir, self_dir_diff)

assert d.same_files == ['file']

assert d.diff_files == ['file2']
expected_report = ['diff {} {}'.format(self_dir, self_dir_diff), "Identical files : ['file']", "Differing files : ['file2']", "Common subdirectories : ['subdir']"]
_assert_report(d.report, expected_report)
print("DirCompareTestCase::test_dircmp: ok")
