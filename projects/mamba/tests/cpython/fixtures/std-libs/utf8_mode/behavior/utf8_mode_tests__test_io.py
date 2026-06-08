# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utf8_mode"
# dimension = "behavior"
# case = "utf8_mode_tests__test_io"
# subject = "cpython.test_utf8_mode.UTF8ModeTests.test_io"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_utf8_mode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_utf8_mode.py::UTF8ModeTests::test_io
"""Auto-ported test: UTF8ModeTests::test_io (CPython 3.12 oracle)."""


import locale
import subprocess
import sys
import textwrap
import unittest
from test import support
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import os_helper, MS_WINDOWS


'\nTest the implementation of the PEP 540: the UTF-8 Mode.\n'

POSIX_LOCALES = ('C', 'POSIX')

VXWORKS = sys.platform == 'vxworks'


# --- test body ---
DEFAULT_ENV = {'PYTHONUTF8': '', 'PYTHONLEGACYWINDOWSFSENCODING': '', 'PYTHONCOERCECLOCALE': '0'}

def _check_io_encoding(module, encoding=None, errors=None):
    filename = __file__
    args = []
    if encoding:
        args.append(f'encoding={encoding!r}')
    if errors:
        args.append(f'errors={errors!r}')
    code = textwrap.dedent('\n            import sys\n            from %s import open\n            filename = sys.argv[1]\n            with open(filename, %s) as fp:\n                print(f"{fp.encoding}/{fp.errors}")\n        ') % (module, ', '.join(args))
    out = get_output('-c', code, filename, PYTHONUTF8='1')
    if not encoding:
        encoding = 'utf-8'
    if not errors:
        errors = 'strict'

    assert out.lower() == f'{encoding}/{errors}'

def check_io_encoding(module):
    _check_io_encoding(module, encoding='latin1')
    _check_io_encoding(module, errors='namereplace')
    _check_io_encoding(module, encoding='latin1', errors='namereplace')

def get_output(*args, failure=False, **kw):
    kw = dict(DEFAULT_ENV, **kw)
    if failure:
        out = assert_python_failure(*args, **kw)
        out = out[2]
    else:
        out = assert_python_ok(*args, **kw)
        out = out[1]
    return out.decode().rstrip('\n\r')

def posix_locale():
    loc = locale.setlocale(locale.LC_CTYPE, None)
    return loc in POSIX_LOCALES
code = textwrap.dedent('\n            import sys\n            filename = sys.argv[1]\n            with open(filename) as fp:\n                print(f"{fp.encoding}/{fp.errors}")\n        ')
filename = __file__
out = get_output('-c', code, filename, PYTHONUTF8='1')

assert out.lower() == 'utf-8/strict'
print("UTF8ModeTests::test_io: ok")
