# /// script
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
