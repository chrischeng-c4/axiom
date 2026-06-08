# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_ioencoding"
# subject = "cpython.test_sys.SysModuleTest.test_ioencoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_ioencoding
"""Auto-ported test: SysModuleTest::test_ioencoding (CPython 3.12 oracle)."""


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
env = dict(os.environ)
env['PYTHONIOENCODING'] = 'cp424'
p = subprocess.Popen([sys.executable, '-c', 'print(chr(0xa2))'], stdout=subprocess.PIPE, env=env)
out = p.communicate()[0].strip()
expected = ('¢' + os.linesep).encode('cp424')

assert out == expected
env['PYTHONIOENCODING'] = 'ascii:replace'
p = subprocess.Popen([sys.executable, '-c', 'print(chr(0xa2))'], stdout=subprocess.PIPE, env=env)
out = p.communicate()[0].strip()

assert out == b'?'
env['PYTHONIOENCODING'] = 'ascii'
p = subprocess.Popen([sys.executable, '-c', 'print(chr(0xa2))'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
out, err = p.communicate()

assert out == b''

assert b'UnicodeEncodeError:' in err

assert b"'\\xa2'" in err
env['PYTHONIOENCODING'] = 'ascii:'
p = subprocess.Popen([sys.executable, '-c', 'print(chr(0xa2))'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
out, err = p.communicate()

assert out == b''

assert b'UnicodeEncodeError:' in err

assert b"'\\xa2'" in err
env['PYTHONIOENCODING'] = ':surrogateescape'
p = subprocess.Popen([sys.executable, '-c', 'print(chr(0xdcbd))'], stdout=subprocess.PIPE, env=env)
out = p.communicate()[0].strip()

assert out == b'\xbd'
print("SysModuleTest::test_ioencoding: ok")
