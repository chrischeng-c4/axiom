# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_implementation"
# subject = "cpython.test_sys.SysModuleTest.test_implementation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_implementation
"""Auto-ported test: SysModuleTest::test_implementation (CPython 3.12 oracle)."""


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
levels = {'alpha': 10, 'beta': 11, 'candidate': 12, 'final': 15}

assert hasattr(sys.implementation, 'name')

assert hasattr(sys.implementation, 'version')

assert hasattr(sys.implementation, 'hexversion')

assert hasattr(sys.implementation, 'cache_tag')
version = sys.implementation.version

assert version[:2] == (version.major, version.minor)
hexversion = version.major << 24 | version.minor << 16 | version.micro << 8 | levels[version.releaselevel] << 4 | version.serial << 0

assert sys.implementation.hexversion == hexversion

assert sys.implementation.name == sys.implementation.name.lower()
print("SysModuleTest::test_implementation: ok")
