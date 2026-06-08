# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "behavior"
# case = "path_like_tests__test_path_commonprefix"
# subject = "cpython.test_genericpath.PathLikeTests.test_path_commonprefix"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericpath.py::PathLikeTests::test_path_commonprefix
"""Auto-ported test: PathLikeTests::test_path_commonprefix (CPython 3.12 oracle)."""


import genericpath
import os
import sys
import unittest
import warnings
from test.support import is_emscripten
from test.support import os_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok
from test.support.os_helper import FakePath


'\nTests common to genericpath, ntpath and posixpath\n'

def create_file(filename, data=b'foo'):
    with open(filename, 'xb', 0) as fp:
        fp.write(data)


# --- test body ---
def assertPathEqual(func):

    assert func(self_file_path) == func(self_file_name)
self_file_name = os_helper.TESTFN
self_file_path = FakePath(os_helper.TESTFN)
pass
create_file(self_file_name, b'test_genericpath.PathLikeTests')

assert os.path.commonprefix([self_file_path, self_file_name]) == self_file_name
print("PathLikeTests::test_path_commonprefix: ok")
