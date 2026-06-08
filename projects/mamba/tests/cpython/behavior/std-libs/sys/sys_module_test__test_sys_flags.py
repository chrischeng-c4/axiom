# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_sys_flags"
# subject = "cpython.test_sys.SysModuleTest.test_sys_flags"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_sys_flags
"""Auto-ported test: SysModuleTest::test_sys_flags (CPython 3.12 oracle)."""


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

assert sys.flags
attrs = ('debug', 'inspect', 'interactive', 'optimize', 'dont_write_bytecode', 'no_user_site', 'no_site', 'ignore_environment', 'verbose', 'bytes_warning', 'quiet', 'hash_randomization', 'isolated', 'dev_mode', 'utf8_mode', 'warn_default_encoding', 'safe_path', 'int_max_str_digits')
for attr in attrs:

    assert hasattr(sys.flags, attr)
    attr_type = bool if attr in ('dev_mode', 'safe_path') else int

    assert type(getattr(sys.flags, attr)) == attr_type

assert repr(sys.flags)

assert len(sys.flags) == len(attrs)

assert sys.flags.utf8_mode in {0, 1, 2}
print("SysModuleTest::test_sys_flags: ok")
