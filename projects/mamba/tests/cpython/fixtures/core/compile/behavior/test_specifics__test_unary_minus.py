# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_unary_minus"
# subject = "cpython.test_compile.TestSpecifics.test_unary_minus"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_unary_minus
"""Auto-ported test: TestSpecifics::test_unary_minus (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
if sys.maxsize == 2147483647:
    all_one_bits = '0xffffffff'

    assert eval(all_one_bits) == 4294967295

    assert eval('-' + all_one_bits) == -4294967295
elif sys.maxsize == 9223372036854775807:
    all_one_bits = '0xffffffffffffffff'

    assert eval(all_one_bits) == 18446744073709551615

    assert eval('-' + all_one_bits) == -18446744073709551615
else:

    raise AssertionError('How many bits *does* this machine have???')

assert isinstance(eval('%s' % (-sys.maxsize - 1)), int)

assert isinstance(eval('%s' % (-sys.maxsize - 2)), int)
print("TestSpecifics::test_unary_minus: ok")
