# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "behavior"
# case = "import_side_effect_tests__test_setting_copyright"
# subject = "cpython.test_site.ImportSideEffectTests.test_setting_copyright"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_site.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_site.py::ImportSideEffectTests::test_setting_copyright
"""Auto-ported test: ImportSideEffectTests::test_setting_copyright (CPython 3.12 oracle)."""


import unittest
import test.support
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import captured_stderr
from test.support.os_helper import TESTFN, EnvironmentVarGuard
import ast
import builtins
import glob
import io
import os
import re
import shutil
import stat
import subprocess
import sys
import sysconfig
import tempfile
import urllib.error
import urllib.request
from unittest import mock
from copy import copy
import site


"Tests for 'site'.\n\nTests assume the initial paths in sys.path once the interpreter has begun\nexecuting have not been removed.\n\n"

if sys.flags.no_site:
    raise unittest.SkipTest('Python was invoked with -S')

HAS_USER_SITE = site.USER_SITE is not None

OLD_SYS_PATH = None

def setUpModule():
    global OLD_SYS_PATH
    OLD_SYS_PATH = sys.path[:]
    if site.ENABLE_USER_SITE and (not os.path.isdir(site.USER_SITE)):
        try:
            os.makedirs(site.USER_SITE)
            site.addsitedir(site.USER_SITE)
        except PermissionError as exc:
            raise unittest.SkipTest('unable to create user site directory (%r): %s' % (site.USER_SITE, exc))

def tearDownModule():
    sys.path[:] = OLD_SYS_PATH

class PthFile(object):
    """Helper class for handling testing of .pth files"""

    def __init__(self, filename_base=TESTFN, imported='time', good_dirname='__testdir__', bad_dirname='__bad'):
        """Initialize instance variables"""
        self.filename = filename_base + '.pth'
        self.base_dir = os.path.abspath('')
        self.file_path = os.path.join(self.base_dir, self.filename)
        self.imported = imported
        self.good_dirname = good_dirname
        self.bad_dirname = bad_dirname
        self.good_dir_path = os.path.join(self.base_dir, self.good_dirname)
        self.bad_dir_path = os.path.join(self.base_dir, self.bad_dirname)

    def create(self):
        """Create a .pth file with a comment, blank lines, an ``import
        <self.imported>``, a line with self.good_dirname, and a line with
        self.bad_dirname.

        Creation of the directory for self.good_dir_path (based off of
        self.good_dirname) is also performed.

        Make sure to call self.cleanup() to undo anything done by this method.

        """
        FILE = open(self.file_path, 'w')
        try:
            print('#import @bad module name', file=FILE)
            print('\n', file=FILE)
            print('import %s' % self.imported, file=FILE)
            print(self.good_dirname, file=FILE)
            print(self.bad_dirname, file=FILE)
        finally:
            FILE.close()
        os.mkdir(self.good_dir_path)

    def cleanup(self, prep=False):
        """Make sure that the .pth file is deleted, self.imported is not in
        sys.modules, and that both self.good_dirname and self.bad_dirname are
        not existing directories."""
        if os.path.exists(self.file_path):
            os.remove(self.file_path)
        if prep:
            self.imported_module = sys.modules.get(self.imported)
            if self.imported_module:
                del sys.modules[self.imported]
        elif self.imported_module:
            sys.modules[self.imported] = self.imported_module
        if os.path.exists(self.good_dir_path):
            os.rmdir(self.good_dir_path)
        if os.path.exists(self.bad_dir_path):
            os.rmdir(self.bad_dir_path)


# --- test body ---
"""Make a copy of sys.path"""
self_sys_path = sys.path[:]

assert hasattr(builtins, 'copyright')

assert hasattr(builtins, 'credits')

assert hasattr(builtins, 'license')
print("ImportSideEffectTests::test_setting_copyright: ok")
