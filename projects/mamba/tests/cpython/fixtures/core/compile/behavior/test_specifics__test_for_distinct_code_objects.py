# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_for_distinct_code_objects"
# subject = "cpython.test_compile.TestSpecifics.test_for_distinct_code_objects"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_for_distinct_code_objects
"""Auto-ported test: TestSpecifics::test_for_distinct_code_objects (CPython 3.12 oracle)."""


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
    f1 = lambda x=1: x
    f2 = lambda x=2: x
    return (f1, f2)
f1, f2 = f()

assert id(f1.__code__) != id(f2.__code__)
print("TestSpecifics::test_for_distinct_code_objects: ok")
