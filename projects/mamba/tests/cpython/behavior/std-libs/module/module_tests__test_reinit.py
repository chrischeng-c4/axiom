# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_reinit"
# subject = "cpython.__init__.ModuleTests.test_reinit"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_reinit
"""Auto-ported test: ModuleTests::test_reinit (CPython 3.12 oracle)."""


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
foo = ModuleType('foo', 'foodocሴ')
foo.bar = 42
d = foo.__dict__
foo.__init__('foo', 'foodoc')

assert foo.__name__ == 'foo'

assert foo.__doc__ == 'foodoc'

assert foo.bar == 42

assert foo.__dict__ == {'__name__': 'foo', '__doc__': 'foodoc', 'bar': 42, '__loader__': None, '__package__': None, '__spec__': None}

assert foo.__dict__ is d
print("ModuleTests::test_reinit: ok")
