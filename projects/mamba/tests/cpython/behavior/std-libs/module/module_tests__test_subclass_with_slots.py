# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_subclass_with_slots"
# subject = "cpython.__init__.ModuleTests.test_subclass_with_slots"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_subclass_with_slots
"""Auto-ported test: ModuleTests::test_subclass_with_slots (CPython 3.12 oracle)."""


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
class ModuleWithSlots(ModuleType):
    __slots__ = ('a', 'b')

    def __init__(self, name):
        super().__init__(name)
m = ModuleWithSlots('name')
try:
    m.a
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
try:
    m.b
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
m.a, m.b = (1, 2)

assert m.a == 1

assert m.b == 2
print("ModuleTests::test_subclass_with_slots: ok")
