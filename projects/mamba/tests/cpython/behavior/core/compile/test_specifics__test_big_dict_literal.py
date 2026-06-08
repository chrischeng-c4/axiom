# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_big_dict_literal"
# subject = "cpython.test_compile.TestSpecifics.test_big_dict_literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_big_dict_literal
"""Auto-ported test: TestSpecifics::test_big_dict_literal (CPython 3.12 oracle)."""


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
dict_size = 65535 + 1
the_dict = '{' + ','.join((f'{x}:{x}' for x in range(dict_size))) + '}'

assert len(eval(the_dict)) == dict_size
print("TestSpecifics::test_big_dict_literal: ok")
