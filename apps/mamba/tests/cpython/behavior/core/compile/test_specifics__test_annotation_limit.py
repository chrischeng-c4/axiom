# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_annotation_limit"
# subject = "cpython.test_compile.TestSpecifics.test_annotation_limit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_annotation_limit
"""Auto-ported test: TestSpecifics::test_annotation_limit (CPython 3.12 oracle)."""


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
s = 'def f(%s): pass'
s %= ', '.join(('a%d:%d' % (i, i) for i in range(300)))
compile(s, '?', 'exec')
print("TestSpecifics::test_annotation_limit: ok")
