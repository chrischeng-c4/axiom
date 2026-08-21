# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_literals_with_leading_zeroes"
# subject = "cpython.test_compile.TestSpecifics.test_literals_with_leading_zeroes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_literals_with_leading_zeroes
"""Auto-ported test: TestSpecifics::test_literals_with_leading_zeroes (CPython 3.12 oracle)."""


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
for arg in ['077787', '0xj', '0x.', '0e', '090000000000000', '080000000000000', '000000000000009', '000000000000008', '0b42', '0BADCAFE', '0o123456789', '0b1.1', '0o4.2', '0b101j', '0o153j', '0b100e1', '0o777e1', '0777', '000777', '000000000000007']:

    try:
        eval(arg)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass

assert eval('0xff') == 255

assert eval('0777.') == 777

assert eval('0777.0') == 777

assert eval('000000000000000000000000000000000000000000000000000777e0') == 777

assert eval('0777e1') == 7770

assert eval('0e0') == 0

assert eval('0000e-012') == 0

assert eval('09.5') == 9.5

assert eval('0777j') == 777j

assert eval('000') == 0

assert eval('00j') == 0j

assert eval('00.0') == 0

assert eval('0e3') == 0

assert eval('090000000000000.') == 90000000000000.0

assert eval('090000000000000.0000000000000000000000') == 90000000000000.0

assert eval('090000000000000e0') == 90000000000000.0

assert eval('090000000000000e-0') == 90000000000000.0

assert eval('090000000000000j') == 90000000000000j

assert eval('000000000000008.') == 8.0

assert eval('000000000000009.') == 9.0

assert eval('0b101010') == 42

assert eval('-0b000000000010') == -2

assert eval('0o777') == 511

assert eval('-0o0000010') == -8
print("TestSpecifics::test_literals_with_leading_zeroes: ok")
