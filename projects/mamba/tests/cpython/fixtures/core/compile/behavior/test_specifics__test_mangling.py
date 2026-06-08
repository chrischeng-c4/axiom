# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_mangling"
# subject = "cpython.test_compile.TestSpecifics.test_mangling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_mangling
"""Auto-ported test: TestSpecifics::test_mangling (CPython 3.12 oracle)."""


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
class A:

    def f():
        __mangled = 1
        __not_mangled__ = 2
        import __mangled_mod
        import __package__.module

assert '_A__mangled' in A.f.__code__.co_varnames

assert '__not_mangled__' in A.f.__code__.co_varnames

assert '_A__mangled_mod' in A.f.__code__.co_varnames

assert '__package__' in A.f.__code__.co_varnames
print("TestSpecifics::test_mangling: ok")
