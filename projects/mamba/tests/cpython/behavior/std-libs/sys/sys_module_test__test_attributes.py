# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_attributes"
# subject = "cpython.test_sys.SysModuleTest.test_attributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_attributes
"""Auto-ported test: SysModuleTest::test_attributes (CPython 3.12 oracle)."""


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

assert isinstance(sys.api_version, int)

assert isinstance(sys.argv, list)
for arg in sys.argv:

    assert isinstance(arg, str)

assert isinstance(sys.orig_argv, list)
for arg in sys.orig_argv:

    assert isinstance(arg, str)

assert sys.byteorder in ('little', 'big')

assert isinstance(sys.builtin_module_names, tuple)

assert isinstance(sys.copyright, str)

assert isinstance(sys.exec_prefix, str)

assert isinstance(sys.base_exec_prefix, str)

assert isinstance(sys.executable, str)

assert len(sys.float_info) == 11

assert sys.float_info.radix == 2

assert len(sys.int_info) == 4

assert sys.int_info.bits_per_digit % 5 == 0

assert sys.int_info.sizeof_digit >= 1

assert sys.int_info.default_max_str_digits >= 500

assert sys.int_info.str_digits_check_threshold >= 100

assert sys.int_info.default_max_str_digits > sys.int_info.str_digits_check_threshold

assert type(sys.int_info.bits_per_digit) == int

assert type(sys.int_info.sizeof_digit) == int

assert isinstance(sys.int_info.default_max_str_digits, int)

assert isinstance(sys.int_info.str_digits_check_threshold, int)

assert isinstance(sys.hexversion, int)

assert len(sys.hash_info) == 9

assert sys.hash_info.modulus < 2 ** sys.hash_info.width
for x in range(1, 100):

    assert pow(x, sys.hash_info.modulus - 1, sys.hash_info.modulus) == 1

assert isinstance(sys.hash_info.inf, int)

assert isinstance(sys.hash_info.nan, int)

assert isinstance(sys.hash_info.imag, int)
algo = sysconfig.get_config_var('Py_HASH_ALGORITHM')
if sys.hash_info.algorithm in {'fnv', 'siphash13', 'siphash24'}:

    assert sys.hash_info.hash_bits in {32, 64}

    assert sys.hash_info.seed_bits in {32, 64, 128}
    if algo == 1:

        assert sys.hash_info.algorithm == 'siphash24'
    elif algo == 2:

        assert sys.hash_info.algorithm == 'fnv'
    elif algo == 3:

        assert sys.hash_info.algorithm == 'siphash13'
    else:

        assert sys.hash_info.algorithm in {'fnv', 'siphash13', 'siphash24'}
else:

    assert algo == 0

assert sys.hash_info.cutoff >= 0

assert sys.hash_info.cutoff < 8

assert isinstance(sys.maxsize, int)

assert isinstance(sys.maxunicode, int)

assert sys.maxunicode == 1114111

assert isinstance(sys.platform, str)

assert isinstance(sys.prefix, str)

assert isinstance(sys.base_prefix, str)

assert isinstance(sys.platlibdir, str)

assert isinstance(sys.version, str)
vi = sys.version_info

assert isinstance(vi[:], tuple)

assert len(vi) == 5

assert isinstance(vi[0], int)

assert isinstance(vi[1], int)

assert isinstance(vi[2], int)

assert vi[3] in ('alpha', 'beta', 'candidate', 'final')

assert isinstance(vi[4], int)

assert isinstance(vi.major, int)

assert isinstance(vi.minor, int)

assert isinstance(vi.micro, int)

assert vi.releaselevel in ('alpha', 'beta', 'candidate', 'final')

assert isinstance(vi.serial, int)

assert vi[0] == vi.major

assert vi[1] == vi.minor

assert vi[2] == vi.micro

assert vi[3] == vi.releaselevel

assert vi[4] == vi.serial

assert vi > (1, 0, 0)

assert isinstance(sys.float_repr_style, str)

assert sys.float_repr_style in ('short', 'legacy')
if not sys.platform.startswith('win'):

    assert isinstance(sys.abiflags, str)
print("SysModuleTest::test_attributes: ok")
