# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_ascii_docstring"
# subject = "cpython.__init__.ModuleTests.test_ascii_docstring"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_ascii_docstring
"""Auto-ported test: ModuleTests::test_ascii_docstring (CPython 3.12 oracle)."""


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
foo = ModuleType('foo', 'foodoc')

assert foo.__name__ == 'foo'

assert foo.__doc__ == 'foodoc'

assert foo.__dict__ == {'__name__': 'foo', '__doc__': 'foodoc', '__loader__': None, '__package__': None, '__spec__': None}
print("ModuleTests::test_ascii_docstring: ok")
