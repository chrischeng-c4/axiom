# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_orig_argv"
# subject = "cpython.test_sys.SysModuleTest.test_orig_argv"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_orig_argv
"""Auto-ported test: SysModuleTest::test_orig_argv (CPython 3.12 oracle)."""


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
code = textwrap.dedent('\n            import sys\n            print(sys.argv)\n            print(sys.orig_argv)\n        ')
args = [sys.executable, '-I', '-X', 'utf8', '-c', code, 'arg']
proc = subprocess.run(args, check=True, capture_output=True, text=True)
expected = [repr(['-c', 'arg']), repr(args)]

assert proc.stdout.rstrip().splitlines() == expected
print("SysModuleTest::test_orig_argv: ok")
