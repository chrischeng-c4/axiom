# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_module_repr_with_filename_only"
# subject = "cpython.__init__.ModuleTests.test_module_repr_with_filename_only"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_module_repr_with_filename_only
"""Auto-ported test: ModuleTests::test_module_repr_with_filename_only (CPython 3.12 oracle)."""


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
del m.__name__
m.__file__ = '/tmp/foo.py'

assert repr(m) == "<module '?' from '/tmp/foo.py'>"
print("ModuleTests::test_module_repr_with_filename_only: ok")
