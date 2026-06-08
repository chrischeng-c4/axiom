# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_is_finalizing"
# subject = "cpython.test_sys.SysModuleTest.test_is_finalizing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_is_finalizing
"""Auto-ported test: SysModuleTest::test_is_finalizing (CPython 3.12 oracle)."""


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

assert sys.is_finalizing() is False
code = 'if 1:\n            import sys\n\n            class AtExit:\n                is_finalizing = sys.is_finalizing\n                print = print\n\n                def __del__(self):\n                    self.print(self.is_finalizing(), flush=True)\n\n            # Keep a reference in the __main__ module namespace, so the\n            # AtExit destructor will be called at Python exit\n            ref = AtExit()\n        '
rc, stdout, stderr = assert_python_ok('-c', code)

assert stdout.rstrip() == b'True'
print("SysModuleTest::test_is_finalizing: ok")
