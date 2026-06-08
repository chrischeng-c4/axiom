# /// script
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
