# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_remove_unused_consts_no_docstring"
# subject = "cpython.test_compile.TestSpecifics.test_remove_unused_consts_no_docstring"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_remove_unused_consts_no_docstring
"""Auto-ported test: TestSpecifics::test_remove_unused_consts_no_docstring (CPython 3.12 oracle)."""


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
def f():
    if True:
        return 'used'
    else:
        return 'unused'

assert f.__code__.co_consts == (None, 'used')
print("TestSpecifics::test_remove_unused_consts_no_docstring: ok")
