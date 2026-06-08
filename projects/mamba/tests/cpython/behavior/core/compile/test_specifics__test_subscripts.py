# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_subscripts"
# subject = "cpython.test_compile.TestSpecifics.test_subscripts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_subscripts
"""Auto-ported test: TestSpecifics::test_subscripts (CPython 3.12 oracle)."""


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
class str_map(object):

    def __init__(self):
        self.data = {}

    def __getitem__(self, key):
        return self.data[str(key)]

    def __setitem__(self, key, value):
        self.data[str(key)] = value

    def __delitem__(self, key):
        del self.data[str(key)]

    def __contains__(self, key):
        return str(key) in self.data
d = str_map()
d[1] = 1

assert d[1] == 1
d[1] += 1

assert d[1] == 2
del d[1]

assert 1 not in d
d[1, 1] = 1

assert d[1, 1] == 1
d[1, 1] += 1

assert d[1, 1] == 2
del d[1, 1]

assert (1, 1) not in d
d[1:2] = 1

assert d[1:2] == 1
d[1:2] += 1

assert d[1:2] == 2
del d[1:2]

assert slice(1, 2) not in d
d[1:2, 1:2] = 1

assert d[1:2, 1:2] == 1
d[1:2, 1:2] += 1

assert d[1:2, 1:2] == 2
del d[1:2, 1:2]

assert (slice(1, 2), slice(1, 2)) not in d
d[1:2:3] = 1

assert d[1:2:3] == 1
d[1:2:3] += 1

assert d[1:2:3] == 2
del d[1:2:3]

assert slice(1, 2, 3) not in d
d[1:2:3, 1:2:3] = 1

assert d[1:2:3, 1:2:3] == 1
d[1:2:3, 1:2:3] += 1

assert d[1:2:3, 1:2:3] == 2
del d[1:2:3, 1:2:3]

assert (slice(1, 2, 3), slice(1, 2, 3)) not in d
d[...] = 1

assert d[...] == 1
d[...] += 1

assert d[...] == 2
del d[...]

assert Ellipsis not in d
d[..., ...] = 1

assert d[..., ...] == 1
d[..., ...] += 1

assert d[..., ...] == 2
del d[..., ...]

assert (Ellipsis, Ellipsis) not in d
print("TestSpecifics::test_subscripts: ok")
