# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_subinterp_intern_dynamically_allocated"
# subject = "cpython.test_sys.SysModuleTest.test_subinterp_intern_dynamically_allocated"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_subinterp_intern_dynamically_allocated
"""Auto-ported test: SysModuleTest::test_subinterp_intern_dynamically_allocated (CPython 3.12 oracle)."""


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
s = 'never interned before' + str(random.randrange(0, 10 ** 9))
t = sys.intern(s)

assert t is s
interp = interpreters.create()
interp.run(textwrap.dedent(f"\n            import sys\n\n            # set `s`, avoid parser interning & constant folding\n            s = str({s.encode()!r}, 'utf-8')\n\n            t = sys.intern(s)\n\n            assert id(t) != {id(s)}, (id(t), {id(s)})\n            assert id(t) != {id(t)}, (id(t), {id(t)})\n            "))
print("SysModuleTest::test_subinterp_intern_dynamically_allocated: ok")
