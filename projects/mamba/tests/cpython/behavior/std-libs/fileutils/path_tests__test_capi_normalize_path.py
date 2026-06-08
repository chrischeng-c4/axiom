# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileutils"
# dimension = "behavior"
# case = "path_tests__test_capi_normalize_path"
# subject = "cpython.test_fileutils.PathTests.test_capi_normalize_path"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileutils.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fileutils.py::PathTests::test_capi_normalize_path
"""Auto-ported test: PathTests::test_capi_normalize_path (CPython 3.12 oracle)."""


import os
import os.path
import unittest
from test.support import import_helper


_testcapi = import_helper.import_module('_testinternalcapi')


# --- test body ---
if os.name == 'nt':
    raise unittest.SkipTest('Windows has its own helper for this')
else:
    from test.test_posixpath import PosixPathTest as posixdata
    tests = posixdata.NORMPATH_CASES
for filename, expected in tests:
    if not os.path.isabs(filename):
        continue
    result = _testcapi.normalize_path(filename)

    assert result == expected
print("PathTests::test_capi_normalize_path: ok")
