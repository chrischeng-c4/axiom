# /// script
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
