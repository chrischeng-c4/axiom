# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_issue20602"
# subject = "cpython.test_sys.SysModuleTest.test_issue20602"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_issue20602
"""Auto-ported test: SysModuleTest::test_issue20602 (CPython 3.12 oracle)."""


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
code = 'if 1:\n            import sys\n            class A:\n                def __del__(self, sys=sys):\n                    print(sys.flags)\n                    print(sys.float_info)\n            a = A()\n            '
rc, out, err = assert_python_ok('-c', code)
out = out.splitlines()

assert b'sys.flags' in out[0]

assert b'sys.float_info' in out[1]
print("SysModuleTest::test_issue20602: ok")
