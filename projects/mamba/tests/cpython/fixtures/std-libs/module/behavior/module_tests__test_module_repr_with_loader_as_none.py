# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_module_repr_with_loader_as_none"
# subject = "cpython.__init__.ModuleTests.test_module_repr_with_loader_as_None"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_module_repr_with_loader_as_None
"""Auto-ported test: ModuleTests::test_module_repr_with_loader_as_None (CPython 3.12 oracle)."""


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
m = ModuleType('foo')
assert m.__loader__ is None

assert repr(m) == "<module 'foo'>"
print("ModuleTests::test_module_repr_with_loader_as_None: ok")
