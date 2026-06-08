# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_module_repr_source"
# subject = "cpython.__init__.ModuleTests.test_module_repr_source"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_module_repr_source
"""Auto-ported test: ModuleTests::test_module_repr_source (CPython 3.12 oracle)."""


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
r = repr(unittest)
starts_with = "<module 'unittest' from '"
ends_with = "__init__.py'>"

assert r[:len(starts_with)] == starts_with

assert r[-len(ends_with):] == ends_with
print("ModuleTests::test_module_repr_source: ok")
