# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_repeated_attribute_pops"
# subject = "cpython.__init__.ModuleTests.test_repeated_attribute_pops"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_repeated_attribute_pops
"""Auto-ported test: ModuleTests::test_repeated_attribute_pops (CPython 3.12 oracle)."""


import importlib.machinery
import unittest
import weakref
from test.support import gc_collect
from test.support import import_helper
from test.support.script_helper import assert_python_ok
import sys


ModuleType = type(sys)

class FullLoader:
    pass

class BareLoader:
    pass


# --- test body ---
m = ModuleType('test')
d = m.__dict__
count = 0
for _ in range(100):
    m.attr = 1
    count += m.attr
    d.pop('attr')

assert count == 100
print("ModuleTests::test_repeated_attribute_pops: ok")
