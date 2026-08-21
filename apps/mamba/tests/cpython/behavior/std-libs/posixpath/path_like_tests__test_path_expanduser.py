# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "behavior"
# case = "path_like_tests__test_path_expanduser"
# subject = "cpython.test_posixpath.PathLikeTests.test_path_expanduser"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_posixpath.py::PathLikeTests::test_path_expanduser
"""Auto-ported test: PathLikeTests::test_path_expanduser (CPython 3.12 oracle)."""


import inspect
import os
import posixpath
import sys
import unittest
from functools import partial
from posixpath import realpath, abspath, dirname, basename, ALLOW_MISSING
from test import support
from test import test_genericpath
from test.support import import_helper
from test.support import cpython_only, os_helper
from test.support.os_helper import FakePath
from unittest import mock


try:
    import posix
except ImportError:
    posix = None

ABSTFN = abspath(os_helper.TESTFN)

def skip_if_ABSTFN_contains_backslash(test):
    """
    On Windows, posixpath.abspath still returns paths with backslashes
    instead of posix forward slashes. If this is the case, several tests
    fail, so skip them.
    """
    found_backslash = '\\' in ABSTFN
    msg = 'ABSTFN is not a posix path - tests fail'
    return [test, unittest.skip(msg)(test)][found_backslash]

def safe_rmdir(dirname):
    try:
        os.rmdir(dirname)
    except OSError:
        pass

def _parameterize(*parameters):
    """Simplistic decorator to parametrize a test

    Runs the decorated test multiple times in subTest, with a value from
    'parameters' passed as an extra positional argument.
    Does *not* call doCleanups() after each run.

    Not for general use. Intended to avoid indenting for easier backports.

    See https://discuss.python.org/t/91827 for discussing generalizations.
    """

    def _parametrize_decorator(func):

        def _parameterized(self, *args, **kwargs):
            for parameter in parameters:
                with self.subTest(parameter):
                    func(self, *args, parameter, **kwargs)
        return _parameterized
    return _parametrize_decorator


# --- test body ---
path = posixpath

def assertPathEqual(func):

    assert func(self_file_path) == func(self_file_name)
self_file_name = os_helper.TESTFN
self_file_path = FakePath(os_helper.TESTFN)
pass
with open(self_file_name, 'xb', 0) as file:
    file.write(b'test_posixpath.PathLikeTests')
assertPathEqual(path.expanduser)
print("PathLikeTests::test_path_expanduser: ok")
