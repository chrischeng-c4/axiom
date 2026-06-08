# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "dir_compare_test_case__test_dircmp_invalid_names"
# subject = "cpython.test_filecmp.DirCompareTestCase.test_dircmp_invalid_names"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_filecmp.py::DirCompareTestCase::test_dircmp_invalid_names
"""Auto-ported test: DirCompareTestCase::test_dircmp_invalid_names (CPython 3.12 oracle)."""


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
for bad_dir, desc in [('\x00', 'NUL bytes dirname'), (f'Top{os.sep}Mid\x00', 'dirname with embedded NUL bytes'), ('\ud834\udd1e', 'surrogate codes (MUSICAL SYMBOL G CLEF)'), ('a' * 1000000, 'very long dirname')]:
    d1 = filecmp.dircmp(self_dir, bad_dir)
    d2 = filecmp.dircmp(bad_dir, self_dir)
    for target in ['left_list', 'right_list', 'left_only', 'right_only', 'common']:
        try:
            getattr(d1, target)
            raise AssertionError('expected (OSError, ValueError)')
        except (OSError, ValueError):
            pass
        try:
            getattr(d2, target)
            raise AssertionError('expected (OSError, ValueError)')
        except (OSError, ValueError):
            pass
print("DirCompareTestCase::test_dircmp_invalid_names: ok")
