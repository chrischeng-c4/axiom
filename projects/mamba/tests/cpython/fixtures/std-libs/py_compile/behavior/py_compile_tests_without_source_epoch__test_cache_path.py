# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "py_compile"
# dimension = "behavior"
# case = "py_compile_tests_without_source_epoch__test_cache_path"
# subject = "cpython.test_py_compile.PyCompileTestsWithoutSourceEpoch.test_cache_path"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_py_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_py_compile.py::PyCompileTestsWithoutSourceEpoch::test_cache_path
"""Auto-ported test: PyCompileTestsWithoutSourceEpoch::test_cache_path (CPython 3.12 oracle)."""


import functools
import importlib.util
import os
import py_compile
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from test import support
from test.support import os_helper, script_helper


def without_source_date_epoch(fxn):
    """Runs function with SOURCE_DATE_EPOCH unset."""

    @functools.wraps(fxn)
    def wrapper(*args, **kwargs):
        with os_helper.EnvironmentVarGuard() as env:
            env.unset('SOURCE_DATE_EPOCH')
            return fxn(*args, **kwargs)
    return wrapper

def with_source_date_epoch(fxn):
    """Runs function with SOURCE_DATE_EPOCH set."""

    @functools.wraps(fxn)
    def wrapper(*args, **kwargs):
        with os_helper.EnvironmentVarGuard() as env:
            env['SOURCE_DATE_EPOCH'] = '123456789'
            return fxn(*args, **kwargs)
    return wrapper

class SourceDateEpochTestMeta(type(unittest.TestCase)):

    def __new__(mcls, name, bases, dct, *, source_date_epoch):
        cls = super().__new__(mcls, name, bases, dct)
        for attr in dir(cls):
            if attr.startswith('test_'):
                meth = getattr(cls, attr)
                if source_date_epoch:
                    wrapper = with_source_date_epoch(meth)
                else:
                    wrapper = without_source_date_epoch(meth)
                setattr(cls, attr, wrapper)
        return cls


# --- test body ---
self_directory = tempfile.mkdtemp(dir=os.getcwd())
self_source_path = os.path.join(self_directory, '_test.py')
self_pyc_path = self_source_path + 'c'
self_cache_path = importlib.util.cache_from_source(self_source_path)
self_cwd_drive = os.path.splitdrive(os.getcwd())[0]
drive = os.path.splitdrive(self_source_path)[0]
if drive:
    os.chdir(drive)
with open(self_source_path, 'w') as file:
    file.write('x = 123\n')
py_compile.compile(self_source_path)

assert os.path.exists(self_cache_path)
print("PyCompileTestsWithoutSourceEpoch::test_cache_path: ok")
