# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "module"
# dimension = "behavior"
# case = "module_tests__test_annotations_getset_raises"
# subject = "cpython.__init__.ModuleTests.test_annotations_getset_raises"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_module/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::ModuleTests::test_annotations_getset_raises
"""Auto-ported test: ModuleTests::test_annotations_getset_raises (CPython 3.12 oracle)."""


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
foo.__annotations__ = {}
del foo.__annotations__
try:
    del foo.__annotations__
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("ModuleTests::test_annotations_getset_raises: ok")
