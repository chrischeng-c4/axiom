# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_dont_clear_dict"
# subject = "cpython.__init__.ModuleTests.test_dont_clear_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_dont_clear_dict
"""Auto-ported test: ModuleTests::test_dont_clear_dict (CPython 3.12 oracle)."""


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
def f():
    foo = ModuleType('foo')
    foo.bar = 4
    return foo
gc_collect()

assert f().__dict__['bar'] == 4
print("ModuleTests::test_dont_clear_dict: ok")
