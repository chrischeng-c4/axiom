# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_remove_empty_basic_block_with_jump_target_label"
# subject = "cpython.test_compile.TestSpecifics.test_remove_empty_basic_block_with_jump_target_label"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_remove_empty_basic_block_with_jump_target_label
"""Auto-ported test: TestSpecifics::test_remove_empty_basic_block_with_jump_target_label (CPython 3.12 oracle)."""


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
def f(x):
    while x:
        0 if 1 else 0
print("TestSpecifics::test_remove_empty_basic_block_with_jump_target_label: ok")
