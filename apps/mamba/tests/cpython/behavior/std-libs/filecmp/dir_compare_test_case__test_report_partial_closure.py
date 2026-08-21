# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "dir_compare_test_case__test_report_partial_closure"
# subject = "cpython.test_filecmp.DirCompareTestCase.test_report_partial_closure"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_filecmp.py::DirCompareTestCase::test_report_partial_closure
"""Auto-ported test: DirCompareTestCase::test_report_partial_closure (CPython 3.12 oracle)."""


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
left_subdir = os.path.join(left_dir, 'subdir')
right_subdir = os.path.join(right_dir, 'subdir')
expected_report = ['diff {} {}'.format(self_dir, self_dir_same), "Identical files : ['file']", "Common subdirectories : ['subdir']", '', 'diff {} {}'.format(left_subdir, right_subdir)]
_assert_report(d.report_partial_closure, expected_report)
print("DirCompareTestCase::test_report_partial_closure: ok")
