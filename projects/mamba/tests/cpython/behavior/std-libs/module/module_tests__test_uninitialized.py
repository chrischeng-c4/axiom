# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_uninitialized"
# subject = "cpython.__init__.ModuleTests.test_uninitialized"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_uninitialized
"""Auto-ported test: ModuleTests::test_uninitialized (CPython 3.12 oracle)."""


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
foo = ModuleType.__new__(ModuleType)

assert isinstance(foo.__dict__, dict)

assert dir(foo) == []
try:
    s = foo.__name__

    raise AssertionError('__name__ = %s' % repr(s))
except AttributeError:
    pass

assert foo.__doc__ == (ModuleType.__doc__ or '')
print("ModuleTests::test_uninitialized: ok")
