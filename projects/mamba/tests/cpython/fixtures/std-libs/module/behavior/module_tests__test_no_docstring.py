# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_no_docstring"
# subject = "cpython.__init__.ModuleTests.test_no_docstring"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_no_docstring
"""Auto-ported test: ModuleTests::test_no_docstring (CPython 3.12 oracle)."""


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
foo = ModuleType('foo')

assert foo.__name__ == 'foo'

assert foo.__doc__ == None

assert foo.__loader__ is None

assert foo.__package__ is None

assert foo.__spec__ is None

assert foo.__dict__ == {'__name__': 'foo', '__doc__': None, '__loader__': None, '__package__': None, '__spec__': None}
print("ModuleTests::test_no_docstring: ok")
