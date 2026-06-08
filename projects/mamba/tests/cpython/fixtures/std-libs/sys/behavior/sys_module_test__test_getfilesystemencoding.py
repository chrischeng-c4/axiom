# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_getfilesystemencoding"
# subject = "cpython.test_sys.SysModuleTest.test_getfilesystemencoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_getfilesystemencoding
"""Auto-ported test: SysModuleTest::test_getfilesystemencoding (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---
def assert_raise_on_new_sys_type(sys_attr):
    arg = sys_attr
    attr_type = type(sys_attr)
    try:
        attr_type(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        attr_type.__new__(attr_type, arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def c_locale_get_error_handler(locale, isolated=False, encoding=None):
    env = os.environ.copy()
    env['LC_ALL'] = locale
    env['PYTHONCOERCECLOCALE'] = '0'
    code = '\n'.join(('import sys', 'def dump(name):', '    std = getattr(sys, name)', '    print("%s: %s" % (name, std.errors))', 'dump("stdin")', 'dump("stdout")', 'dump("stderr")'))
    args = [sys.executable, '-X', 'utf8=0', '-c', code]
    if isolated:
        args.append('-I')
    if encoding is not None:
        env['PYTHONIOENCODING'] = encoding
    else:
        env.pop('PYTHONIOENCODING', None)
    p = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env, universal_newlines=True)
    stdout, stderr = p.communicate()
    return stdout

def check_fsencoding(fs_encoding, expected=None):

    assert fs_encoding is not None
    codecs.lookup(fs_encoding)
    if expected:

        assert fs_encoding == expected

def check_locale_surrogateescape(locale):
    out = c_locale_get_error_handler(locale, isolated=True)

    assert out == 'stdin: surrogateescape\nstdout: surrogateescape\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding=':ignore')

    assert out == 'stdin: ignore\nstdout: ignore\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding='iso8859-1')

    assert out == 'stdin: strict\nstdout: strict\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding='iso8859-1:')

    assert out == 'stdin: strict\nstdout: strict\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding=':')

    assert out == 'stdin: surrogateescape\nstdout: surrogateescape\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding='')

    assert out == 'stdin: surrogateescape\nstdout: surrogateescape\nstderr: backslashreplace\n'
fs_encoding = sys.getfilesystemencoding()
if sys.platform == 'darwin':
    expected = 'utf-8'
else:
    expected = None
check_fsencoding(fs_encoding, expected)
print("SysModuleTest::test_getfilesystemencoding: ok")
