# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_clear_dict_in_ref_cycle"
# subject = "cpython.__init__.ModuleTests.test_clear_dict_in_ref_cycle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_clear_dict_in_ref_cycle
"""Auto-ported test: ModuleTests::test_clear_dict_in_ref_cycle (CPython 3.12 oracle)."""


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
destroyed = []
m = ModuleType('foo')
m.destroyed = destroyed
s = 'class A:\n    def __init__(self, l):\n        self.l = l\n    def __del__(self):\n        self.l.append(1)\na = A(destroyed)'
exec(s, m.__dict__)
del m
gc_collect()

assert destroyed == [1]
print("ModuleTests::test_clear_dict_in_ref_cycle: ok")
